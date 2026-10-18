// Retained rigid-part recipes shared by land and naval miniatures. +x forward, +y up.
const DARK='#26333a',IRON='#788b95';
function materialFor(color){
 if(color==='primary'||color==='secondary'||color==='accent')return 'cloth';
 if(['#788b95','#26333a','#7f7767','#777c83','#aebba1'].includes(color))return 'iron';
 if(['#785239','#594332','#493e34','#72523b','#654d37','#533b2f','#62402f','#6c4b35'].includes(color))return 'wood';
 if(['#dab866','#be9b58','#b89a63','#c19b62'].includes(color))return 'brass';
 if(['#a3fff0','#9fe8db','#9cccd3','#bdfcf4'].includes(color))return 'glass';
 if(['#e6dcc2','#d8d5b6','#d5cdbb'].includes(color))return 'cloth';
 return 'organic';
}
export class Sculpt {
 constructor(){this.parts=[];}
 part(shape,at,size,color,motion='',pivot=at,rotation=[0,0,0],amplitude=.5){const p={shape,at,size,color,motion,pivot,rotation,amplitude,on:[],material:materialFor(color),detail:0};if(motion==='kneeL'||motion==='kneeR')p.kneeDrop=.32;this.parts.push(p);return p;}
 box(at,size,c,m='',pivot=at,rot=[0,0,0]){return this.part('box',at,size,c,m,pivot,rot);}
 orb(at,size,c,m='',pivot=at){return this.part('ellipsoid',at,size,c,m,pivot);}
 // The renderer keeps these as the same cheap retained ellipsoids as heads and wheels.
 // Use them for living bodies and fairings; boxes are reserved for planks, armour plates,
 // and machinery so a unit reads as sculpted instead of assembled from blocks.
 capsule(at,size,c,m='',pivot=at,rot=[0,0,0]){const p=this.part('capsule',at,size,c,m,pivot,rot);p.rounded=true;return p;}
 taper(a,b,r,c,m='',pivot=a){
  const dx=b[0]-a[0],dy=b[1]-a[1],dz=b[2]-a[2],len=Math.hypot(dx,dy,dz);
  return this.capsule(a.map((x,i)=>(x+b[i])/2),[r*1.45,len+r*.35,r*1.45],c,m,pivot,[Math.atan2(dz,Math.hypot(dx,dy)),0,-Math.atan2(dx,dy)]);
 }
 poly(at,points,depth,c,m='',pivot=at,rot=[0,0,0]){const p=this.part('poly',at,[.1,.1,depth],c,m,pivot,rot);p.points=points;return p;}
 // Closed oval sections around the local Y axis: [height, x radius, z radius].
 // Authored profiles form robes, helmets and organic trunks without box extrusion.
 lathe(at,profile,c,m='',pivot=at,rot=[0,0,0]){const p=this.part('lathe',at,[1,1,1],c,m,pivot,rot);p.profile=profile;return p;}
 // Closed longitudinal hull: rings are [x, height, half-beam].  This stays an
 // internal retained primitive; callers cannot submit it through model normalization.
 loft(rings,c,pivot=[0,0,0]){const p=this.part('loft',pivot,[1,1,1],c,'',pivot);p.rings=rings;return p;}
 rod(a,b,r,c,m='',pivot=a){const dx=b[0]-a[0],dy=b[1]-a[1],dz=b[2]-a[2],len=Math.hypot(dx,dy,dz);return this.box(a.map((x,i)=>(x+b[i])/2),[r,len,r],c,m,pivot,[Math.atan2(dz,Math.hypot(dx,dy)),0,-Math.atan2(dx,dy)]);}
 wheel(at,r,c){
  const wheel=this.part('cylinder',at,[r*2,r*2,.13],c,'roll',at,[0,0,0],.4),z=at[2]+Math.sign(at[2]||1)*.075;
  this.part('cylinder',[at[0],at[1],z],[r*1.55,r*1.55,.025],c===DARK?IRON:DARK,'roll',at);
  for(const a of [0,Math.PI/2])this.box([at[0],at[1],z+Math.sign(z)*.02],[r*1.7,.035,.035],c,'roll',at,[0,0,a]);
  return wheel;
 }
 // Metadata stays beside the retained primitive rather than changing its geometry
 // contract. The renderer can drop ornament and rotate coherent subassemblies.
 tag(start,{material,detail,joint,aim,pivot}={}){for(const p of this.parts.slice(start)){if(material)p.material=material;if(detail!==undefined)p.detail=detail;if(joint)p.joint=joint;if(aim)p.aim=aim;if(pivot)p.pivot=[...pivot];}}
 transform(start,scale,offset){for(const p of this.parts.slice(start)){p.at=p.at.map((x,i)=>x*scale+offset[i]);p.size=p.size.map(x=>x*scale);p.pivot=p.pivot.map((x,i)=>x*scale+offset[i]);if(p.kneeDrop)p.kneeDrop*=scale;if(p.points)p.points=p.points.map(x=>x*scale);if(p.profile)p.profile=p.profile.map(r=>r.map(x=>x*scale));}}
}
