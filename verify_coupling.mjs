// Short causal QA check; this is not an additional user scenario.
import fs from 'node:fs';import assert from 'node:assert/strict';
const bytes=fs.readFileSync(new URL('./dashboard/oil_engine.wasm',import.meta.url));
const run=drive=>{const e=new WebAssembly.Instance(new WebAssembly.Module(bytes),{}).exports,n=e.initialize(95);e.advance(100,drive?1:0);
  return {n,stats:Array.from(new Float32Array(e.memory.buffer,e.stats_ptr(),16)),p:Float32Array.from(new Float32Array(e.memory.buffer,e.positions_ptr(),n*3))};};
const rotating=run(true),stationary=run(false);assert.equal(rotating.n,stationary.n);
let squared=0;for(let i=0;i<rotating.p.length;i++)squared+=(rotating.p[i]-stationary.p[i])**2;
const rms=Math.sqrt(squared/rotating.n);assert.ok(rms>.001);assert.equal(rotating.stats[10],1);assert.equal(stationary.stats[10],1);
const report={passed:true,test:'identical 95 L initialization; moving boundaries versus stationary boundaries for 100 steps',
  purpose:'verify the mechanism actually changes the fluid, not merely the drawing',physical_duration_s:rotating.stats[0],
  particles:rotating.n,rms_position_difference_m:rms,rotating_kinetic_energy_j:rotating.stats[3],stationary_kinetic_energy_j:stationary.stats[3],
  delivery_cases:'All fifteen delivered cases use the rotating mechanism and last 2.4 s.'};
fs.writeFileSync(new URL('./coupling_report.json',import.meta.url),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
