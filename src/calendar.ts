function iso(date: Date) { return date.toISOString().slice(0, 10); }

export function moveMonth(month: string, offset: number) {
    const date = new Date(month + '-01T12:00:00Z');
    date.setUTCMonth(date.getUTCMonth() + offset);
    return iso(date).slice(0, 7);
}

export function monthDays(month: string) {
    const first = new Date(month + '-01T12:00:00Z');
    first.setUTCDate(1 - (first.getUTCDay() + 6) % 7);
    return Array.from({ length: 42 }, (_, index) => {
        const day = new Date(first);
        day.setUTCDate(first.getUTCDate() + index);
        return iso(day);
    });
}

export function compareTasks(a:{due:string;priority:string;done:boolean;completedAt?:number|null}, b:typeof a, filter:string, today:string) {
    if(filter==='done')return (b.completedAt??0)-(a.completedAt??0)||b.due.localeCompare(a.due);
    const group=(date:string)=>date<today?0:date===today?1:2;
    const priorities=['alta','media','baixa'];
    return Number(a.done)-Number(b.done)
        || (filter==='overview'?group(a.due)-group(b.due):0)
        || priorities.indexOf(a.priority)-priorities.indexOf(b.priority)
        || a.due.localeCompare(b.due);
}

export function brazilianDate(date:string) {
    return /^\d{4}-\d{2}-\d{2}$/.test(date)?date.split('-').reverse().join('/'):'';
}
export function parseBrazilianDate(value:string):string|null {
    if(!/^\d{2}\/\d{2}\/\d{4}$/.test(value))return null;
    const date=value.split('/').reverse().join('-');
    const parsed=new Date(date+'T12:00:00Z');
    return date.slice(0,4)!=='0000'&&!Number.isNaN(parsed.getTime())&&parsed.toISOString().slice(0,10)===date?date:null;
}

export function calendarDisplayDate(date:string,start:string|null,timezone:number) {
    if(start===null)return date;
    return new Date(new Date(date+'T'+start+':00Z').getTime()-(timezone+180)*60000).toISOString().slice(0,10);
}
