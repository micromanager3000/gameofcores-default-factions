// Retained rigid-part recipes shared by land and naval miniatures. +x forward, +y up.
const DARK='#26333a',IRON='#788b95';
export class Sculpt {
 constructor(){this.parts=[];}
 part(shape,at,size,color,motion='',pivot=at,rotation=[0,0,0],amplitude=.5){const p={shape,at,size,color,motion,pivot,rotation,amplitude,on:[]};this.parts.push(p);return p;}
 box(at,size,c,m='',pivot=at,rot=[0,0,0]){return this.part('box',at,size,c,m,pivot,rot);}
 orb(at,size,c,m='',pivot=at){return this.part('ellipsoid',at,size,c,m,pivot);}
 poly(at,points,depth,c,m='',pivot=at,rot=[0,0,0]){const p=this.part('poly',at,[.1,.1,depth],c,m,pivot,rot);p.points=points;return p;}
 rod(a,b,r,c,m='',pivot=a){const dx=b[0]-a[0],dy=b[1]-a[1],dz=b[2]-a[2],len=Math.hypot(dx,dy,dz);return this.box(a.map((x,i)=>(x+b[i])/2),[r,len,r],c,m,pivot,[Math.atan2(dz,Math.hypot(dx,dy)),0,-Math.atan2(dx,dy)]);}
 wheel(at,r,c){
  const wheel=this.part('cylinder',at,[r*2,r*2,.13],c,'roll',at,[0,0,0],.4),z=at[2]+Math.sign(at[2]||1)*.075;
  this.part('cylinder',[at[0],at[1],z],[r*1.55,r*1.55,.025],c===DARK?IRON:DARK,'roll',at);
  for(const a of [0,Math.PI/2])this.box([at[0],at[1],z+Math.sign(z)*.02],[r*1.7,.035,.035],c,'roll',at,[0,0,a]);
  return wheel;
 }
 transform(start,scale,offset){for(const p of this.parts.slice(start)){p.at=p.at.map((x,i)=>x*scale+offset[i]);p.size=p.size.map(x=>x*scale);p.pivot=p.pivot.map((x,i)=>x*scale+offset[i]);if(p.points)p.points=p.points.map(x=>x*scale);}}
}
