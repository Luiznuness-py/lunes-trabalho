import { useEffect, useId, useRef } from 'react';
import type { ReactNode } from 'react';

export function ConfirmDelete({ title, description, busy, error, onConfirm, onCancel, confirmLabel = 'Confirmar exclusão', busyLabel = 'Excluindo…', children, confirmDisabled = false }: {
    title: string;
    description: string;
    busy: boolean;
    error: string;
    onConfirm: () => void;
    onCancel: () => void;
    confirmLabel?:string;
    busyLabel?:string;
    children?: ReactNode;
    confirmDisabled?: boolean;
}) {
    const dialog = useRef<HTMLDialogElement>(null);
    const titleId = useId();
    const descriptionId = useId();
    useEffect(() => {
        const element = dialog.current!;
        const previous = document.activeElement as HTMLElement | null;
        element.showModal();
        return () => { element.close(); if (previous?.isConnected) previous.focus(); };
    }, []);
    return <dialog ref={dialog} className="confirm-delete" aria-labelledby={titleId} aria-describedby={descriptionId}
        onCancel={event => { event.preventDefault(); if (!busy) onCancel(); }}>
        <h2 id={titleId}>{title}</h2>
        <p id={descriptionId}>{description}</p>
        {children}
        {error && <p role="alert" className="aviso">{error}</p>}
        <div className="confirm-delete-actions">
            <button type="button" autoFocus disabled={busy} onClick={onCancel}>Cancelar</button>
            <button type="button" className="confirm-delete-submit" disabled={busy || confirmDisabled} onClick={onConfirm}>{busy ? busyLabel : confirmLabel}</button>
        </div>
    </dialog>;
}
