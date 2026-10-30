import { useEffect, useRef, useState, type InputHTMLAttributes } from 'react';
import { brazilianDate, parseBrazilianDate } from './calendar';

type Props = Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'value' | 'onChange'> & {value:string;onChange:(value:string)=>void};
export function DateInput({value,onChange,min,max,...props}:Props) {
 const [text,setText]=useState(()=>brazilianDate(value));
 const input=useRef<HTMLInputElement>(null);
 useEffect(()=>{setText(brazilianDate(value));},[value]);
 useEffect(()=>{const date=parseBrazilianDate(text);input.current?.setCustomValidity(text&&(!date||(min&&date<String(min))||(max&&date>String(max)))?'Informe uma data válida no formato dd/mm/aaaa, dentro do período permitido.':'');},[text,min,max]);
 return <input {...props} ref={input} type="text" inputMode="numeric" placeholder="dd/mm/aaaa" maxLength={10} value={text} onChange={e=>{
  const next=e.target.value;setText(next);const date=parseBrazilianDate(next);
  const valid=date!==null&&(!min||date>=String(min))&&(!max||date<=String(max));
  e.target.setCustomValidity(next&&!valid?'Informe uma data válida no formato dd/mm/aaaa, dentro do período permitido.':'');
  if(!next||valid)onChange(date??'');
 }}/>;
}
