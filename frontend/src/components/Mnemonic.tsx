// Highlights the first character of a label in the accent color and dims
// the rest - the bracket-mnemonic look TUI dashboards use for panel/menu
// titles (e.g. the reference dashboard's "Clock" / "Weather" headers).
export function Mnemonic({ text }: { text: string }) {
  if (text.length === 0) return null

  return (
    <span className="uppercase tracking-widest">
      <span className="text-[var(--accent)]">{text[0]}</span>
      <span className="text-[var(--text-dim)]">{text.slice(1)}</span>
    </span>
  )
}
