import fs from 'node:fs';import path from 'node:path';import {fileURLToPath} from 'node:url';import {spawn} from 'node:child_process';
import {FIXED,VOLUMES,SENSOR_COUNTS} from './dashboard/constants.js';
const ROOT=path.dirname(fileURLToPath(import.meta.url));
const cases=VOLUMES.flatMap(oil_liters=>SENSOR_COUNTS.map(sensor_count=>({id:`oil_${oil_liters}_s${sensor_count}`,oil_liters,sensor_count,label:`${oil_liters} L · ${sensor_count} sensores`,constants:FIXED})));
fs.mkdirSync(path.join(ROOT,'data'),{recursive:true});fs.writeFileSync(path.join(ROOT,'data/cases.json'),JSON.stringify(cases,null,2));
function run(liters){return new Promise((resolve,reject)=>{const dir=path.join(ROOT,'data/volumes',String(liters));fs.mkdirSync(dir,{recursive:true});const log=fs.openSync(path.join(dir,'run.log'),'w');const child=spawn(process.execPath,[path.join(ROOT,'run_case.mjs'),String(liters)],{cwd:ROOT,stdio:['ignore',log,log]});child.on('error',reject);child.on('exit',code=>{fs.closeSync(log);console.log(`${liters} L: ${code===0?'aceptado':'rechazado'}`);resolve({liters,code});});});}
const volumes=process.argv.slice(2).map(Number);const result=await Promise.all((volumes.length?volumes:VOLUMES).map(run));
if(result.some(x=>x.code!==0))process.exitCode=1;
