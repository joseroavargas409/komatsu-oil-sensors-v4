import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import {fileURLToPath} from 'node:url';
import {FIXED} from './dashboard/constants.js';import {SensorArray} from './dashboard/sensor-model.js';
const ROOT=path.dirname(fileURLToPath(import.meta.url));
const statsKeys=['time_s','particles','mass_kg','kinetic_energy_j','max_speed_m_s','max_native_speed_m_s',
 'max_projection_m','projected_particle_steps','max_solid_penetration','outside_particles','finite','oil_top_z_m',
 'splash_particles','initial_surface_z_m','particle_radius_m','motor_omega_rad_s'];
const bodyKeys=['index','stage','role','center_y','inner','outer','length','teeth','inward','phase','orbit','speed_factor','orbit_factor'];
const round=(x,n=6)=>Number(x.toFixed(n));
export async function computeCase(liters,{output,drive=true}={}) {
 if(!Number.isFinite(liters)||liters<30||liters>145)throw new Error('Volumen permitido: 30 a 145 L.');
 const start=performance.now(),bytes=fs.readFileSync(path.join(ROOT,'dashboard/oil_engine.wasm'));
 const module=new WebAssembly.Module(bytes);if(WebAssembly.Module.imports(module).length)throw new Error('El módulo debe ser autónomo, sin bibliotecas nativas.');
 const e=new WebAssembly.Instance(module,{}).exports,n=e.initialize(liters),sh=crypto.createHash('sha256').update(bytes).digest('hex');
 const readStats=()=>Object.fromEntries(Array.from(new Float32Array(e.memory.buffer,e.stats_ptr(),16)).map((x,i)=>[statsKeys[i],round(x,8)]));
 const geometry=Array.from({length:e.body_count()},(_,i)=>Object.fromEntries(Array.from(new Float32Array(e.memory.buffer,e.geometry_ptr()+i*13*4,13)).map((v,j)=>[bodyKeys[j],v])));
 const steps=20,frames=121,positions=new Float32Array(n*3*frames),sensor=new SensorArray(liters/1000/n),samples=[],times=[];
 let maxOutside=0,minFinite=1,last;
 for(let frame=0;frame<frames;frame++) {
   if(frame)e.advance(steps,drive?1:0);
   const data=new Float32Array(e.memory.buffer,e.positions_ptr(),n*3);positions.set(data,frame*n*3);
   const st=readStats();last=st;times.push(st.time_s);maxOutside=Math.max(maxOutside,st.outside_particles);minFinite=Math.min(minFinite,st.finite);
   samples.push({time_s:st.time_s,metrics:st,sensors:sensor.sample(st.time_s,data)});
   if(frame%20===0)console.log(JSON.stringify({progress:frame/(frames-1),physical_s:st.time_s,wall_s:round((performance.now()-start)/1000,1)}));
   if(!st.finite)throw new Error('Estado no finito: caso rechazado.');
 }
 const checks={duration_at_least_2_s:last.time_s>=2,finite:minFinite===1,mass_preserved:Math.abs(last.mass_kg-liters/1000*FIXED.density_kg_m3)<1e-4,
  particles_preserved:last.particles===n,contained:maxOutside===0,solid_free:last.max_solid_penetration===0,
  bounded_projection:last.max_projection_m<=FIXED.numerical_projection_limit_m,bounded_speed:last.max_native_speed_m_s<20,
  autonomous_wasm:WebAssembly.Module.imports(module).length===0};
 const summary={oil_liters:liters,particles:n,mass_kg:last.mass_kg,duration_s:last.time_s,frames,
  engine:'Salva 0.10.0 · DFSPH · custom SI laminar viscosity',engine_sha256:sh,
  numerical_checks:checks,numerically_accepted:Object.values(checks).every(Boolean),wall_clock_s:(performance.now()-start)/1000,
  max_projection_m:last.max_projection_m,max_native_speed_m_s:last.max_native_speed_m_s,
  final_kinetic_energy_j:last.kinetic_energy_j,initial_level_mm:(samples[0].metrics.initial_surface_z_m+.5)*1000,
  fluid_particle_order:'stable indices; Salva z-sort disabled in pinned upstream revision',drive,constants:FIXED};
 const data={summary,constants:FIXED,times,samples,geometry,positions_file:'positions.bin',particle_stride:3,
  probes_model:'non-intrusive local occupancy, first-order response, deterministic per-channel noise and hysteresis'};
 if(output){fs.mkdirSync(output,{recursive:true});fs.writeFileSync(path.join(output,'positions.bin'),Buffer.from(positions.buffer));fs.writeFileSync(path.join(output,'result.json'),JSON.stringify(data));fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2));}
 if(!summary.numerically_accepted)throw new Error('Controles numéricos rechazados: '+JSON.stringify(checks));
 return data;
}
if(process.argv[1]===fileURLToPath(import.meta.url)) {
 const liters=Number(process.argv[2]||95),out=process.argv[3]||path.join(ROOT,'data/volumes',String(liters));
 try{const data=await computeCase(liters,{output:out,drive:!process.argv.includes('--stopped')});console.log(JSON.stringify(data.summary));}catch(e){console.error(e.stack||e.message);process.exitCode=1;}
}
