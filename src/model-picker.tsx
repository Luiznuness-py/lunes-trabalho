import { useEffect, useId, useRef, useState } from 'react';
import { Check, ChevronDown } from 'lucide-react';

type Option = { id: string; nome: string };
export function ModelPicker({ id, options, value, disabled, onChange, placeholder = 'Escolha um modelo' }: { id: string; options: Option[]; value: string | null; disabled: boolean; placeholder?: string; onChange: (value: string) => void }) {
  const menuId = useId();
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const items = useRef<(HTMLButtonElement | null)[]>([]);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const selected = options.find(option => option.id === value);
  function close(focus = false) {
    menu.current?.hidePopover();
    if (focus) trigger.current?.focus();
  }
  function focus(index: number) {
    setActive(index);
    items.current[index]?.focus();
  }
  function show(index = Math.max(0, options.findIndex(option => option.id === value))) {
    if (disabled || !options.length || !trigger.current || !menu.current) return;
    const rect = trigger.current.getBoundingClientRect();
    const below = innerHeight - rect.bottom - 12;
    const above = rect.top - 12;
    Object.assign(menu.current.style, {
      width: `${Math.min(rect.width, innerWidth - 24)}px`,
      left: `${Math.max(12, Math.min(rect.left, innerWidth - rect.width - 12))}px`,
      top: below >= above ? `${rect.bottom + 6}px` : 'auto',
      bottom: below >= above ? 'auto' : `${innerHeight - rect.top + 6}px`,
      maxHeight: `${Math.max(44, Math.min(320, Math.max(below, above)))}px`,
    });
    menu.current.showPopover();
    focus(index);
  }
  useEffect(() => { if (disabled) close(); }, [disabled]);
  useEffect(() => {
    if (!open) return;
    const dismiss = () => close();
    const scroll = (event: Event) => { if (event.target !== menu.current) close(); };
    window.addEventListener('resize', dismiss);
    document.addEventListener('scroll', scroll, true);
    return () => { window.removeEventListener('resize', dismiss); document.removeEventListener('scroll', scroll, true); };
  }, [open]);
  return <>
    <button id={id} type="button" className="modelo-trigger" ref={trigger} disabled={disabled || !options.length} aria-labelledby={`${id}-label ${id}`} aria-haspopup="listbox" aria-controls={menuId} aria-expanded={open}
      onClick={() => open ? close() : show()}
      onKeyDown={event => {
        if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
          event.preventDefault();
          show(event.key === 'ArrowUp' || event.key === 'End' ? options.length - 1 : 0);
        }
      }}><span>{selected?.nome ?? (value ? `${value} (indisponível)` : placeholder)}</span><ChevronDown size={16} aria-hidden="true" /></button>
    <div id={menuId} ref={menu} popover="auto" role="listbox" aria-labelledby={`${id}-label`} className="modelo-menu" onToggle={event => setOpen(event.currentTarget.matches(':popover-open'))}
      onKeyDown={event => {
        if (event.key === 'Escape') { event.preventDefault(); close(true); }
        else if (event.key === 'Tab') close(true);
        else if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
          event.preventDefault();
          focus(event.key === 'Home' ? 0 : event.key === 'End' ? options.length - 1 : (active + (event.key === 'ArrowDown' ? 1 : -1) + options.length) % options.length);
        } else if (event.key.length === 1 && /\S/.test(event.key)) {
          const index = options.findIndex((option, i) => i > active && option.nome.toLocaleLowerCase().startsWith(event.key.toLocaleLowerCase()));
          const fallback = options.findIndex(option => option.nome.toLocaleLowerCase().startsWith(event.key.toLocaleLowerCase()));
          if (index >= 0 || fallback >= 0) { event.preventDefault(); focus(index >= 0 ? index : fallback); }
        }
      }}>{options.map((option,index) => <button key={option.id} type="button" role="option" aria-selected={option.id === value} tabIndex={index === active ? 0 : -1} ref={element => { items.current[index] = element; }} onFocus={() => setActive(index)} onClick={() => { onChange(option.id); close(true); }}><span>{option.nome}</span>{option.id === value && <Check size={16} aria-hidden="true" />}</button>)}</div>
  </>;
}
