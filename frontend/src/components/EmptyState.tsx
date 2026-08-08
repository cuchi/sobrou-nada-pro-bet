import type { ReactNode } from 'react';

/**
 * Shared empty-state visual: an inline SVG glyph, a short title, and an
 * optional hint line. Used wherever a list-of-N can render zero items so
 * the user always sees something intentional instead of a one-liner.
 *
 * The default icons are pre-drawn SVGs sized to `1.5em` × `1.5em` and
 * inherit `currentColor`, so the surrounding CSS controls the tint.
 */
export type EmptyIcon = 'trophy' | 'groups' | 'ball' | 'ticket' | 'search';

interface Props {
  icon?: EmptyIcon;
  title: string;
  hint?: ReactNode;
  className?: string;
}

const ICONS: Record<EmptyIcon, ReactNode> = {
  // Sad/cracked trophy — same shape as the brand logo (small version).
  trophy: (
    <svg viewBox="4 6 16 14" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M 7 7 L 17 7 L 16 14 Q 15.8 15 14.8 15 L 9.2 15 Q 8.2 15 8 14 Z" />
      <path d="M 7 8.4 Q 5 8.6 5 10.4 Q 5 12 7 12.4" />
      <path d="M 17 8.4 Q 19 8.6 19 10.4 Q 19 12 17 12.4" />
      <path d="M 12 15 L 12 17.6" />
      <path d="M 9 19 L 15 19 M 9.6 17.6 L 14.4 17.6" />
      <path d="M 14 7.6 L 13 9.4 L 14.4 11 L 13.2 13 L 14.6 14.6" />
    </svg>
  ),
  // Three overlapping circles — friends in a group.
  groups: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true">
      <circle cx="9" cy="9" r="3.2" />
      <circle cx="16.5" cy="10.5" r="2.4" />
      <path d="M 4 19 Q 4 14 9 14 Q 14 14 14 19" strokeLinecap="round" />
      <path d="M 14.5 19 Q 14.5 15.5 17.5 15.5 Q 20.5 15.5 20.5 19" strokeLinecap="round" />
    </svg>
  ),
  // Football (soccer ball) — filled truncated icosahedron silhouette, in
// currentColor so the surrounding CSS controls the tint.
  ball: (
    <svg viewBox="0 0 512 512" fill="currentColor" aria-hidden="true">
      <path d="M255.03 33.813c-1.834-.007-3.664-.007-5.5.03-6.73.14-13.462.605-20.155 1.344.333.166.544.32.47.438L204.78 75.063l73.907 49.437-.125.188 70.625.28L371 79.282 342.844 52c-15.866-6.796-32.493-11.776-49.47-14.78-12.65-2.24-25.497-3.36-38.343-3.407zM190.907 88.25l-73.656 36.78-13.813 98.407 51.344 33.657 94.345-43.438 14.875-76.5-73.094-48.906zm196.344.344l-21.25 44.5 36.75 72.72 62.063 38.905 11.312-21.282c.225.143.45.403.656.75-.77-4.954-1.71-9.893-2.81-14.782-6.446-28.59-18.59-55.962-35.5-79.97-9.07-12.872-19.526-24.778-31.095-35.5l-20.125-5.342zm-302.656 23c-6.906 8.045-13.257 16.56-18.938 25.5-15.676 24.664-26.44 52.494-31.437 81.312C31.783 232.446 30.714 246.73 31 261l20.25 5.094 33.03-40.5L98.75 122.53l-14.156-10.936zm312.719 112.844l-55.813 44.75-3.47 101.093 39.626 21.126 77.188-49.594 4.406-78.75-.094.157-61.844-38.783zm-140.844 6.406l-94.033 43.312-1.218 76.625 89.155 57.376 68.938-36.437 3.437-101.75-66.28-39.126zm-224.22 49.75c.91 8.436 2.29 16.816 4.156 25.094 6.445 28.59 18.62 55.96 35.532 79.968 3.873 5.5 8.02 10.805 12.374 15.938l-9.374-48.156.124-.032-27.03-68.844-15.782-3.968zm117.188 84.844l-51.532 8.156 10.125 52.094c8.577 7.49 17.707 14.332 27.314 20.437 14.612 9.287 30.332 16.88 46.687 22.594l62.626-13.69-4.344-31.124-90.875-58.47zm302.437.5l-64.22 41.25-42 47.375 4.408 6.156c12.027-5.545 23.57-12.144 34.406-19.72 23.97-16.76 44.604-38.304 60.28-62.97 2.51-3.947 4.87-7.99 7.125-12.092zm-122.78 97.656l-79.94 9.625-25.968 5.655c26.993 4 54.717 3.044 81.313-2.813 9.412-2.072 18.684-4.79 27.75-8.062l-3.156-4.406z"/>
    </svg>
  ),
  // Bet ticket / receipt — wide line with notches.
  ticket: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round" aria-hidden="true">
      <path d="M 3 7 L 21 7 L 21 11 L 19 11 L 19 13 L 21 13 L 21 17 L 3 17 L 3 13 L 5 13 L 5 11 L 3 11 Z" />
      <path d="M 8 11 L 8 13" />
      <path d="M 12 11 L 12 13" />
      <path d="M 16 11 L 16 13" />
    </svg>
  ),
  // Magnifying glass with empty interior — for search/no-results.
  search: (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" aria-hidden="true">
      <circle cx="10.5" cy="10.5" r="6" />
      <path d="M 15 15 L 20 20" />
    </svg>
  ),
};

export default function EmptyState({ icon = 'trophy', title, hint, className }: Props) {
  const cls = className ? `empty-state ${className}` : 'empty-state';
  return (
    <div className={cls} role="status">
      <span className="empty-state-icon">{ICONS[icon]}</span>
      <p className="empty-state-title">{title}</p>
      {hint && <p className="empty-state-hint">{hint}</p>}
    </div>
  );
}