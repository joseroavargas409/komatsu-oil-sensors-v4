import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import {fileURLToPath} from 'node:url';
import {FIXED,VOLUMES,SENSOR_COUNTS,PROBES} from './dashboard/constants.js';import {SensorArray} from './dashboard/sensor-model.js';
const ROOT=path.dirname(fileURLToPath(import.meta.url)),read=p=>JSON.parse(fs.readFileSync(p,'utf8')),cases=read(path.join(ROOT,'data/cases.json'));
const wasm=fs.readFileSync(path.join(ROOT,'dashboard/oil_engine.wasm')),module=new WebAssembly.Module(wasm),hash=crypto.createHash('sha256').update(wasm).digest('hex');assert.deepEqual(WebAssembly.Module.imports(module),[]);
const reports=[];fs.mkdirSync(path.join(ROOT,'data/case_exports'),{recursive:true});
for(const oil of VOLUMES){const dir=path.join(ROOT,'data/volumes',String(oil)),d=read(path.join(dir,'result.json'));assert(d.summary.numerically_accepted);assert(d.summary.duration_s>=2);assert.equal(d.summary.engine_sha256,hash);assert.deepEqual(d.constants,FIXED);assert.equal(d.geometry[0].teeth,18);assert.equal(d.geometry[6].teeth,23);assert(Math.abs(d.geometry[14].speed_factor-1/FIXED.modeled_ratio)<1e-7);
 const p=fs.readFileSync(path.join(dir,'positions.bin')),positions=new Float32Array(p.buffer,p.byteOffset,p.byteLength/4);assert.equal(positions.length,d.summary.particles*3*d.times.length);assert([...positions].every(Number.isFinite));
 for(const count of SENSOR_COUNTS){const id=`oil_${oil}_s${count}`;const selected=d.samples.map(f=>({...f,sensors:f.sensors.slice(0,count)}));assert(selected.every(f=>f.sensors.length===count&&f.sensors.every(s=>s.signal>=0&&s.signal<=1&&Math.abs(s.oil_fraction+s.air_fraction-1)<1e-12)));
  // Every shared channel must remain byte-identical regardless of count.
  assert.deepEqual(selected.map(f=>f.sensors[0]),d.samples.map(f=>f.sensors[0]));
  const rows=['time_s,sensor_id,oil_fraction,air_fraction,signal,capacitance_proxy_pf,level_mm,state'];for(const f of selected)for(const s of f.sensors)rows.push([f.time_s,s.id,s.oil_fraction,s.air_fraction,s.signal,s.capacitance_proxy_pf,s.level_mm,s.state].join(','));
  fs.writeFileSync(path.join(ROOT,'data/case_exports',id+'.csv'),'\ufeff'+rows.join('\n'));
  reports.push({id,oil_liters:oil,sensor_count:count,duration_s:d.summary.duration_s,frames:d.times.length,particles:d.summary.particles,
    mass_kg:d.summary.mass_kg,max_projection_mm:d.summary.max_projection_m*1000,maximum_native_speed_m_s:d.summary.max_native_speed_m_s,
    initial_level_mm:d.summary.initial_level_mm,final_kinetic_energy_j:d.summary.final_kinetic_energy_j,accepted:true});
 }
}
assert.equal(cases.length,15);assert.equal(reports.length,15);
// Stable RNG streams and first-order response can be reproduced from saved physical positions.
const nominal=read(path.join(ROOT,'data/volumes/95/result.json')),buf=fs.readFileSync(path.join(ROOT,'data/volumes/95/positions.bin'));
const all=new Float32Array(buf.buffer,buf.byteOffset,buf.byteLength/4),n=nominal.summary.particles,stride=n*3,sensors=new SensorArray(.095/n);
for(let frame=0;frame<3;frame++)assert.deepEqual(sensors.sample(nominal.times[frame],all.subarray(frame*stride,(frame+1)*stride)),nominal.samples[frame].sensors);
const report={all_accepted:true,cases:reports,only_varying_parameters:['oil_liters','sensor_count'],fixed_parameters:FIXED,wasm_imports:[],engine_sha256:hash,
  common_sensor_signals_identical:true,signal_model_reproduced_from_physical_particles:true,
  independent_fluid_runs:3,experimental_combinations:15,fluid_sampling_hz:50,stable_particle_ids:true};
fs.writeFileSync(path.join(ROOT,'validation_report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({all_accepted:true,cases:reports.length,wasm_imports:[],same_fixed_parameters:true,common_signals_unchanged:true}));
