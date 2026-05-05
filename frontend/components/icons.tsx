export function MagnusLogo({ size = 32 }: { size?: number }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 32 32" shapeRendering="geometricPrecision">
      <path d="M 16 0 C 7 0, 0 7, 0 16 C 0 25, 7 32, 16 32 C 25 32, 32 25, 32 16 C 32 7, 25 0, 16 0 Z" fill="#E8752C" />
      <ellipse cx="16" cy="20" rx="10" ry="8" fill="#FBE8C8" />
      <g transform="translate(16 17.5)">
        <path d="M -9 -7 L -6 -12 L -3.5 -7 Z" fill="#C25812" />
        <path d="M  9 -7 L  6 -12 L  3.5 -7 Z" fill="#C25812" />
        <path d="M -9 -4 C -9 -8, -6 -10, 0 -10 C 6 -10, 9 -8, 9 -4 C 9.5 4, 6 8, 0 8 C -6 8, -9.5 4, -9 -4 Z" fill="#E8752C" />
        <path d="M -2.5 -8 q 0 4, 1 6 M 0 -8 q 0 4, 0 6 M 2.5 -8 q 0 4, -1 6" stroke="#9F4A0E" strokeWidth="1.1" fill="none" strokeLinecap="round" opacity="0.7" />
        <circle cx="-3.5" cy="-2" r="1.6" fill="#100A06" />
        <circle cx=" 3.5" cy="-2" r="1.6" fill="#100A06" />
        <circle cx="-3.1" cy="-2.4" r="0.5" fill="#FFFFFF" />
        <circle cx=" 3.9" cy="-2.4" r="0.5" fill="#FFFFFF" />
        <ellipse cx="0" cy="3" rx="3.5" ry="2.5" fill="#FBE8C8" />
        <path d="M -1.2 1.6 C -1.2 0.6, 1.2 0.6, 1.2 1.6 C 1.2 2.6, 0 3.6, 0 3.6 C 0 3.6, -1.2 2.6, -1.2 1.6 Z" fill="#E68A78" />
        <path d="M 0 3.6 L 0 5 M -1.5 5.6 Q 0 6.4, 1.5 5.6" stroke="#5A3520" strokeWidth="0.6" fill="none" strokeLinecap="round" />
      </g>
    </svg>
  );
}

export function PawIcon({ size = 16, style }: { size?: number; style?: React.CSSProperties }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor" style={style}>
      <ellipse cx="8" cy="11" rx="3.2" ry="2.6" />
      <circle cx="4" cy="6" r="1.5" />
      <circle cx="8" cy="4.5" r="1.5" />
      <circle cx="12" cy="6" r="1.5" />
    </svg>
  );
}

export function GearIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9c.17.39.51.71.92.87.2.08.42.13.64.13H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function PlusIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
      <path d="M12 5v14M5 12h14" />
    </svg>
  );
}

export function SendIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5 12l14-7-4 14-3-6-7-1z" />
    </svg>
  );
}

export function SearchIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <circle cx="11" cy="11" r="7" />
      <path d="M21 21l-4.3-4.3" />
    </svg>
  );
}

export function SunIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
    </svg>
  );
}

export function MoonIcon({ size = 14 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}
