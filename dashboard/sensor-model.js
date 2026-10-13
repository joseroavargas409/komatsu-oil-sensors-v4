import {FIXED,PROBES} from './constants.js';
export function kernel(r,h){const q=r/h;if(q>=1)return 0;return 8/(Math.PI*h**3)*(q<=.5?1+6*(q**3-q**2):2*(1-q)**3);}
export class SensorArray {
  constructor(particleVolume){this.volume=particleVolume;this.h=4*FIXED.particle_radius_m;this.lastTime=null;this.filtered=new Float64Array(PROBES.length);this.wet=new Array(PROBES.length).fill(false);this.seeds=PROBES.map((_,i)=>(FIXED.sensor_seed+(i+1)*2654435761)>>>0);const spacing=2*FIXED.particle_radius_m;let norm=0;for(let x=-3;x<=3;x++)for(let y=-3;y<=3;y++)for(let z=-3;z<=3;z++)norm+=kernel(Math.hypot(x,y,z)*spacing,this.h)*this.volume;this.norm=norm;}
  random(i){let a=this.seeds[i];a^=a<<13;a^=a>>>17;a^=a<<5;this.seeds[i]=a>>>0;return ((a>>>0)+.5)/4294967296;}
  sample(time,positions){
    const locations=PROBES.map(s=>s.position),raw=new Float64Array(PROBES.length),levels=new Float64Array(PROBES.length);const spacing=.032;
    // Occupancy and vertical column integral at actual probe coordinates.
    // Spatial hashing makes this independent of the selected sensor count.
    const grid=new Map(),h=this.h;
    for(let j=0;j<positions.length;j+=3){const x=positions[j],y=positions[j+1],z=positions[j+2],key=`${Math.floor(x/h)},${Math.floor(y/h)},${Math.floor(z/h)}`;if(!grid.has(key))grid.set(key,[]);grid.get(key).push(j);}
    const occupancy=q=>{let sum=0;const base=q.map(v=>Math.floor(v/h));for(let x=-1;x<=1;x++)for(let y=-1;y<=1;y++)for(let z=-1;z<=1;z++){const ids=grid.get(`${base[0]+x},${base[1]+y},${base[2]+z}`)||[];for(const j of ids)sum+=kernel(Math.hypot(positions[j]-q[0],positions[j+1]-q[1],positions[j+2]-q[2]),h)*this.volume;}return Math.min(1,Math.max(0,sum/this.norm));};
    for(let i=0;i<locations.length;i++){const q=locations[i];raw[i]=occupancy(q);for(let z=-.48;z<=.48;z+=spacing)if(Math.hypot(q[0],z)<.5)levels[i]+=occupancy([q[0],q[1],z])*spacing*1000;}
    const dt=this.lastTime===null?0:Math.max(0,time-this.lastTime),alpha=this.lastTime===null?1:1-Math.exp(-dt/FIXED.sensor_tau_s);this.lastTime=time;
    return PROBES.map((s,i)=>{this.filtered[i]+=alpha*(raw[i]-this.filtered[i]);const noise=Math.sqrt(-2*Math.log(this.random(i)))*Math.cos(2*Math.PI*this.random(i))*FIXED.sensor_noise_std;const signal=Math.max(0,Math.min(1,this.filtered[i]+noise));this.wet[i]=signal>.6?true:signal<.4?false:this.wet[i];return {id:s.id,oil_fraction:raw[i],air_fraction:1-raw[i],signal,level_mm:levels[i],capacitance_proxy_pf:20*(1+1.2*signal),wet:this.wet[i],state:signal<.2?'dry':signal<.8?'mixed':'wet',temperature_c:FIXED.temperature_c};});
  }
}
