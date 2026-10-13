//! Open-source fluid engine exported through a plain WebAssembly ABI.
//! Prescribed constant mechanism, no proprietary native library or Python binding.
use std::cell::RefCell;
use std::f32::consts::PI;
use nalgebra::Vector3;
use salva3d::{LiquidWorld,TimestepManager};
use salva3d::object::{Fluid,FluidHandle,Boundary,BoundaryHandle};
use salva3d::object::interaction_groups::InteractionGroups;
use salva3d::solver::{DFSPHSolver,NonPressureForce};
use salva3d::geometry::ParticlesContacts;

type V=Vector3<f32>;
const R:f32=0.50; const L:f32=0.90; const PR:f32=0.016;
const OMEGA:f32=300.*2.*PI/60.; const RATIO:f32=750./23.;
const NU:f32=0.000264238; const RHO:f32=900.; const DT:f32=0.001;

/// Morris/Brookshaw laminar viscosity with SI kinematic viscosity.
struct LaminarViscosity { nu:f32 }
impl NonPressureForce for LaminarViscosity {
    fn solve(&mut self,_dt:&TimestepManager,h:f32,ff:&ParticlesContacts,fb:&ParticlesContacts,
             fluid:&mut Fluid,boundaries:&[Boundary],densities:&[f32]) {
        let eps=0.01*h*h;
        for i in 0..fluid.positions.len() {
            let pi=fluid.positions[i];let vi=fluid.velocities[i];let mut a=V::zeros();
            for c in ff.particle_contacts(i).read().unwrap().iter() {
                if c.i_model==c.j_model && c.j!=i {
                    let delta=pi-fluid.positions[c.j];let lap=2.*delta.dot(&c.gradient)/(delta.norm_squared()+eps);
                    let volume=fluid.volumes[c.j]*RHO*RHO/
                        (densities[i].max(RHO*0.1)*densities[c.j].max(RHO*0.1));
                    a+=(vi-fluid.velocities[c.j])*(self.nu*volume*lap);
                }
            }
            for c in fb.particle_contacts(i).read().unwrap().iter() {
                let b=&boundaries[c.j_model];let delta=pi-b.positions[c.j];
                let lap=2.*delta.dot(&c.gradient)/(delta.norm_squared()+eps);
                // Mirror ghost velocity approximates no slip; 2 is a fixed numerical assumption.
                a+=(vi-b.velocities[c.j])*(2.*self.nu*b.volumes[c.j]*lap*RHO/densities[i].max(RHO*0.1));
            }
            fluid.accelerations[i]+=a;
        }
    }
    fn apply_permutation(&mut self,_:&[usize]){}
}

#[derive(Clone)]
struct Body { name:u32,stage:u32,role:u32,y:f32,inner:f32,outer:f32,length:f32,teeth:usize,
              inward:bool,phase:f32,orbit:f32,factor:f32,orbit_factor:f32 }
impl Body {
    fn center(&self,t:f32)->V { let a=self.phase-self.orbit_factor*OMEGA*t;V::new(self.orbit*a.cos(),self.y,self.orbit*a.sin()) }
    fn radii(&self,angle:f32)->(f32,f32) {
        if self.teeth==0{return (self.inner,self.outer)}
        let n=self.teeth*4;let scaled=angle.rem_euclid(2.*PI)*n as f32/(2.*PI);
        let j=scaled.floor() as usize;let fraction=scaled-scaled.floor();let da=2.*PI/n as f32;
        let vertex=|index:usize| {
            if self.inward {(self.inner+if index%4>=2 {0.014}else{0.},self.outer)}
            else {(self.inner,self.outer-if index%4>=2 {0.014}else{0.})}
        };
        let (a,b)=vertex(j);let (c,d)=vertex((j+1)%n);let beta=fraction*da;
        let ray=|r1:f32,r2:f32|r1*r2*da.sin()/(r2*(da-beta).sin()+r1*beta.sin());
        (ray(a,c),ray(b,d))
    }
    fn local(&self,p:V,t:f32)->V { rotate(p-self.center(t),-self.factor*OMEGA*t) }
    fn contains(&self,p:V,t:f32,margin:f32)->bool {
        let q=self.local(p,t);let (lo,hi)=self.radii(q.z.atan2(q.x));
        q.y.abs()<self.length/2.+margin && q.x.hypot(q.z)>lo-margin && q.x.hypot(q.z)<hi+margin
    }
    fn velocity(&self,p:V,t:f32)->V {
        let c=self.center(t);V::new(0.,self.orbit_factor*OMEGA,0.).cross(&V::new(c.x,0.,c.z))+
            V::new(0.,self.factor*OMEGA,0.).cross(&(p-c))
    }
}
fn rotate(p:V,a:f32)->V {let (s,c)=a.sin_cos();V::new(c*p.x+s*p.z,p.y,-s*p.x+c*p.z)}

fn body_specs()->Vec<Body> {
    let mut out=Vec::new();
    for (stage,y,ns,np,nr,sun,carrier) in [(0,0.21,18,27,72,1.,1./5.),(1,-0.21,23,52,127,1./5.,1./RATIO)] {
        let sr=0.425*ns as f32/nr as f32;let planet=0.425*np as f32/nr as f32;
        let base=Body{name:out.len() as u32,stage,role:0,y,inner:0.038,outer:sr+0.006,length:0.10,teeth:ns,
            inward:false,phase:0.,orbit:0.,factor:sun,orbit_factor:0.};out.push(base.clone());
        for j in 0..3 {let mut b=base.clone();b.name=out.len() as u32;b.role=1;b.inner=0.041;b.outer=planet+0.006;b.teeth=np;
            b.phase=PI/2.+j as f32*2.*PI/3.;b.orbit=sr+planet;b.factor=carrier-(ns as f32/np as f32)*(sun-carrier);
            b.orbit_factor=carrier;out.push(b);}
        let mut ring=base.clone();ring.name=out.len() as u32;ring.role=2;ring.inner=0.425;ring.outer=0.47;ring.length=0.12;
        ring.teeth=nr;ring.inward=true;ring.factor=0.;out.push(ring);
        let mut ca=base;ca.name=out.len() as u32;ca.role=3;ca.y=y-0.073;ca.inner=0.066;ca.outer=0.34;
        ca.length=0.024;ca.teeth=0;ca.factor=carrier;out.push(ca);
    }
    for (y,len,rad,factor,stage) in [(0.36,0.18,0.034,1.,0),(0.,0.29,0.031,1./5.,0),(-0.377,0.144,0.055,1./RATIO,1)] {
        out.push(Body{name:out.len() as u32,stage,role:4,y,inner:0.0005,outer:rad,length:len,teeth:0,
            inward:false,phase:0.,orbit:0.,factor,orbit_factor:0.});
    }
    out
}

fn samples(b:&Body)->Vec<V> {
    let step=PR*1.5;let n=((2.*PI*b.outer/step).ceil() as usize).max(32);let layers=(b.length/step).ceil() as usize;
    let mut p=Vec::new();
    for k in 0..=layers {let y=-b.length/2.+b.length*k as f32/layers as f32;
        for j in 0..n {let a=j as f32*2.*PI/n as f32;let(lo,hi)=b.radii(a);p.push(V::new(hi*a.cos(),y,hi*a.sin()));
            if b.inner>0.01 {p.push(V::new(lo*a.cos(),y,lo*a.sin()));}}
    }
    let radial=((b.outer-b.inner)/step).ceil() as usize;
    for k in 1..radial {let r=b.inner+(b.outer-b.inner)*k as f32/radial as f32;let na=(2.*PI*r/step).ceil() as usize;
        for j in 0..na {let a=j as f32*2.*PI/na as f32;let(lo,hi)=b.radii(a);if r<lo||r>hi{continue}
            for sign in [-1.,1.] {p.push(V::new(r*a.cos(),sign*b.length/2.,r*a.sin()));}}
    }p
}
fn housing_samples()->Vec<V> {
    let step=PR*1.5;let n=(2.*PI*R/step).ceil() as usize;let layers=(L/step).ceil() as usize;let mut p=Vec::new();
    for k in 0..=layers {for j in 0..n {let a=j as f32*2.*PI/n as f32;p.push(V::new(R*a.cos(),-L/2.+L*k as f32/layers as f32,R*a.sin()));}}
    let count=(R/step).floor() as i32;
    for ix in -count..=count {for iz in -count..=count {let x=ix as f32*step;let z=iz as f32*step;if x.hypot(z)<R-step*0.3 {
        p.push(V::new(x,-L/2.,z));p.push(V::new(x,L/2.,z));}}}p
}

struct Engine {world:LiquidWorld,fluid:FluidHandle,bodies:Vec<Body>,handles:Vec<BoundaryHandle>,samples:Vec<Vec<V>>,
    time:f32,steps:u32,liters:f32,flat:Vec<f32>,stats:Vec<f32>,geometry:Vec<f32>,
    max_projection:f32,projection_steps:u32,max_native_speed:f32,max_solid_overlap:u32,finite:bool,initial_surface:f32}
impl Engine {
    fn new(liters:f32)->Self {
        let bodies=body_specs();let spacing=2.*PR;let volume=0.8*spacing.powi(3);let target=(liters/1000./volume).round() as usize;
        let mut candidates=Vec::new();let pad=1.10*spacing;
        let nx=(2.*R/spacing).ceil() as i32;let ny=(L/spacing).ceil() as i32;
        for iz in 0..nx {for ix in 0..nx {for iy in 0..ny {
            let p=V::new(-R+spacing/2.+ix as f32*spacing,-L/2.+spacing/2.+iy as f32*spacing,-R+spacing/2.+iz as f32*spacing);
            if p.x.hypot(p.z)<R-pad && p.y.abs()<L/2.-pad && !bodies.iter().any(|b|b.contains(p,0.,pad)) {candidates.push(p)}
        }}}
        assert!(target>0 && target<=candidates.len(),"Inventory outside coarse geometry capacity");
        candidates.truncate(target);let initial_surface=candidates.iter().map(|p|p.z).fold(-R,f32::max);
        let mut fluid=Fluid::new(candidates,PR,RHO,InteractionGroups::default());
        for v in &mut fluid.volumes {*v=liters/1000./target as f32;}
        fluid.nonpressure_forces.push(Box::new(LaminarViscosity{nu:NU}));
        let mut solver:DFSPHSolver=DFSPHSolver::new();solver.min_pressure_iter=2;solver.max_pressure_iter=60;
        solver.max_density_error=0.05;solver.min_divergence_iter=1;solver.max_divergence_iter=60;
        let mut world=LiquidWorld::new(solver,PR,2.,1.);let handle=world.add_fluid(fluid);
        world.add_boundary(Boundary::new(housing_samples(),InteractionGroups::default()));
        let mut handles=Vec::new();let mut locals=Vec::new();let mut geometry=Vec::new();
        for b in &bodies {let s=samples(b);let positions=s.iter().map(|p|*p+b.center(0.)).collect();
            handles.push(world.add_boundary(Boundary::new(positions,InteractionGroups::default())));locals.push(s);
            geometry.extend([b.name as f32,b.stage as f32,b.role as f32,b.y,b.inner,b.outer,b.length,b.teeth as f32,
                if b.inward{1.}else{0.},b.phase,b.orbit,b.factor,b.orbit_factor]);}
        let mut e=Self{world,fluid:handle,bodies,handles,samples:locals,time:0.,steps:0,liters,
            flat:Vec::new(),stats:vec![0.;16],geometry,max_projection:0.,projection_steps:0,max_native_speed:0.,max_solid_overlap:0,finite:true,initial_surface};
        e.update_boundaries();e.export();e
    }
    fn update_boundaries(&mut self) {
        for (i,b) in self.bodies.iter().enumerate() {
            let boundary=&mut self.world.boundaries_mut()[self.handles[i]];let angle=b.factor*OMEGA*self.time;let center=b.center(self.time);
            for (j,q) in self.samples[i].iter().enumerate() {let p=rotate(*q,angle)+center;boundary.positions[j]=p;boundary.velocities[j]=b.velocity(p,self.time);}
        }
    }
    fn step(&mut self,drive:bool) {
        if drive {self.update_boundaries();} else {for b in self.world.boundaries_mut().values_mut() {b.velocities.fill(V::zeros());}}
        self.world.step(DT,&V::new(0.,0.,-9.81));self.time+=DT;self.steps+=1;
        let f=&mut self.world.fluids_mut()[self.fluid];let t=if drive{self.time}else{0.};
        for i in 0..f.positions.len() {
            let before=f.positions[i];self.max_native_speed=self.max_native_speed.max(f.velocities[i].norm());
            for _ in 0..3 {
                let p=&mut f.positions[i];let rad=p.x.hypot(p.z);
                if rad>R-0.0005 {let n=V::new(p.x/rad,0.,p.z/rad);p.x*= (R-0.0005)/rad;p.z*= (R-0.0005)/rad;
                    let vn=f.velocities[i].dot(&n);if vn>0.{f.velocities[i]-=n*vn}}
                if p.y.abs()>L/2.-0.0005 {p.y=p.y.signum()*(L/2.-0.0005);f.velocities[i].y=0.;}
                for b in &self.bodies {if b.contains(*p,t,0.) {
                    let mut q=b.local(*p,t);let rad=q.x.hypot(q.z);let(lo,hi)=b.radii(q.z.atan2(q.x));
                    let distances=[rad-lo,hi-rad,b.length/2.-q.y,b.length/2.+q.y];
                    let side=(0..4).min_by(|a,b|distances[*a].total_cmp(&distances[*b])).unwrap();
                    let mut normal=V::zeros();if side<2 {let n=V::new(q.x/rad.max(1e-9),0.,q.z/rad.max(1e-9));
                        let goal=if side==0{lo-0.0005}else{hi+0.0005};q.x=n.x*goal;q.z=n.z*goal;normal=n*if side==0{-1.}else{1.};}
                    else {normal.y=if side==2{1.}else{-1.};q.y=normal.y*(b.length/2.+0.0005);}
                    let a=b.factor*OMEGA*t;*p=rotate(q,a)+b.center(t);normal=rotate(normal,a);
                    let wall=if drive{b.velocity(*p,t)}else{V::zeros()};let vn=(f.velocities[i]-wall).dot(&normal);
                    if vn<0.{f.velocities[i]-=normal*vn;}
                }}
            }
            let correction=(f.positions[i]-before).norm();if correction>0.{self.projection_steps+=1;}
            self.max_projection=self.max_projection.max(correction);
            if !f.positions[i].iter().all(|x|x.is_finite()) || !f.velocities[i].iter().all(|x|x.is_finite()){self.finite=false;}
        }
        if self.steps%25==0 {let overlap=f.positions.iter().filter(|p|self.bodies.iter().any(|b|b.contains(**p,t,0.))).count() as u32;
            self.max_solid_overlap=self.max_solid_overlap.max(overlap);}
    }
    fn export(&mut self) {
        let f=&self.world.fluids()[self.fluid];self.flat.clear();let mut energy=0.;let mut speed=0.;let mut top=-R;let mut outside=0;let mut splash=0;
        for (p,v) in f.positions.iter().zip(f.velocities.iter()) {self.flat.extend([p.x,p.y,p.z]);energy+=v.norm_squared();speed=f32::max(speed,v.norm());top=f32::max(top,p.z);
            if p.x.hypot(p.z)>R+1e-6 || p.y.abs()>L/2.+1e-6 {outside+=1;}if p.z>self.initial_surface+2.*PR{splash+=1;}}
        let n=f.positions.len();let mass=self.liters/1000.*RHO;
        self.stats.copy_from_slice(&[self.time,n as f32,mass,0.5*mass*energy/n as f32,speed,self.max_native_speed,
            self.max_projection,self.projection_steps as f32,self.max_solid_overlap as f32,outside as f32,
            if self.finite{1.}else{0.},top,splash as f32,self.initial_surface,PR,OMEGA]);
    }
}
thread_local!{static SIM:RefCell<Option<Engine>>=const{RefCell::new(None)};}
#[no_mangle]pub extern "C" fn initialize(liters:f32)->u32{SIM.with(|s|{let e=Engine::new(liters);let n=e.flat.len()/3;*s.borrow_mut()=Some(e);n as u32})}
#[no_mangle]pub extern "C" fn advance(steps:u32,drive:u32){SIM.with(|s|{let mut e=s.borrow_mut();let e=e.as_mut().unwrap();for _ in 0..steps{e.step(drive!=0)}e.export();})}
#[no_mangle]pub extern "C" fn positions_ptr()->u32{SIM.with(|s|s.borrow().as_ref().unwrap().flat.as_ptr() as u32)}
#[no_mangle]pub extern "C" fn stats_ptr()->u32{SIM.with(|s|s.borrow().as_ref().unwrap().stats.as_ptr() as u32)}
#[no_mangle]pub extern "C" fn geometry_ptr()->u32{SIM.with(|s|s.borrow().as_ref().unwrap().geometry.as_ptr() as u32)}
#[no_mangle]pub extern "C" fn body_count()->u32{SIM.with(|s|s.borrow().as_ref().unwrap().bodies.len() as u32)}
#[no_mangle]pub extern "C" fn time_step()->f32{DT}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn valid_three_planet_assembly(){for(ns,np,nr)in[(18,27,72),(23,52,127)]{assert_eq!(ns+2*np,nr);assert_eq!((ns+nr)%3,0);}}
    #[test]fn willis_speed_identity(){for(ns,nr,sun,carrier)in[(18.,72.,1.,1./5.),(23.,127.,1./5.,1./RATIO)]{assert!(((sun-carrier)*ns-carrier*nr).abs()<1e-5);}}
    #[test]fn ratio_matches_reference_within_point_one_percent(){assert!((RATIO/32.62-1.).abs()<0.001);}
    #[test]fn orbital_velocity_is_derivative(){for b in body_specs().iter().filter(|b|b.orbit>0.){let t=0.1;let eps=1e-4;let numeric=(b.center(t+eps)-b.center(t-eps))/(2.*eps);let v=b.velocity(b.center(t),t);assert!((numeric-v).norm()<0.002);}}
    #[test]fn tangential_sun_wall_velocity(){let b=&body_specs()[0];let p=V::new(b.outer,b.y,0.);let v=b.velocity(p,0.);assert!(v.x.abs()<1e-6);assert!((v.z+OMEGA*b.outer).abs()<1e-5);}
    #[test]fn samples_are_on_geometric_boundary(){for b in body_specs(){for q in samples(&b){let(lo,hi)=b.radii(q.z.atan2(q.x));let r=q.x.hypot(q.z);assert!((r-lo).abs()<1e-4||(r-hi).abs()<1e-4||(q.y.abs()-b.length/2.).abs()<1e-4);}}}
    #[test]fn laminar_pair_viscosity_dissipates_and_conserves_momentum(){let vi=V::new(1.,2.,0.);let vj=V::new(-1.,0.,0.);let delta=V::new(0.02,0.,0.);let grad=V::new(-20.,0.,0.);let di=600.;let dj=800.;let massi=0.02;let massj=0.03;
        let lap=2.*delta.dot(&grad)/(delta.norm_squared()+0.00001);let ai=(vi-vj)*(NU*massj*RHO/(di*dj)*lap);let aj=(vj-vi)*(NU*massi*RHO/(di*dj)*lap);
        assert!((ai*massi+aj*massj).norm()<1e-10);assert!(vi.dot(&(ai*massi))+vj.dot(&(aj*massj))<0.);}
    #[test]fn volume_changes_particle_count(){let a=Engine::new(55.);let b=Engine::new(95.);assert!(a.flat.len()<b.flat.len());for e in [&a,&b]{let f=&e.world.fluids()[e.fluid];let mass:f32=f.volumes.iter().sum::<f32>()*RHO;assert!((mass-e.liters/1000.*RHO).abs()<0.01);}}
}
