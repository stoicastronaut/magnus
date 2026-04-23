type MascotVariant = "awake" | "sleepy" | "breathing";

interface MascotProps {
  size?: number;
  variant?: MascotVariant;
  style?: React.CSSProperties;
}

export function Mascot({ size = 160, variant = "awake", style = {} }: MascotProps) {
  const classes = [
    "magnus",
    variant === "sleepy" && "magnus--sleepy",
    "magnus--blinking",
    variant === "breathing" && "magnus--breathing",
  ].filter(Boolean).join(" ");

  return (
    <div className={classes} style={{ "--mascot-size": `${size}px`, ...style } as React.CSSProperties}>
      <div className="magnus__body">
        <div className="magnus__stripe magnus__stripe--b1" />
        <div className="magnus__stripe magnus__stripe--b2" />
        <div className="magnus__stripe magnus__stripe--b3" />
        <div className="magnus__stripe magnus__stripe--b4" />
        <div className="magnus__bib" />
        <div className="magnus__paw magnus__paw--l" />
        <div className="magnus__paw magnus__paw--r" />
      </div>
      <div className="magnus__tail" />
      <div className="magnus__head">
        <div className="magnus__ear magnus__ear--l" />
        <div className="magnus__ear magnus__ear--r" />
        <div className="magnus__ear-inner magnus__ear-inner--l" />
        <div className="magnus__ear-inner magnus__ear-inner--r" />
        <div className="magnus__m" />
        <div className="magnus__eye magnus__eye--l" />
        <div className="magnus__eye magnus__eye--r" />
        <div className="magnus__muzzle" />
        <div className="magnus__nose" />
        <div className="magnus__mouth" />
        <div className="magnus__whisker magnus__whisker--l1" />
        <div className="magnus__whisker magnus__whisker--l2" />
        <div className="magnus__whisker magnus__whisker--r1" />
        <div className="magnus__whisker magnus__whisker--r2" />
      </div>
    </div>
  );
}

export function PawBullet({ style }: { style?: React.CSSProperties }) {
  return <span className="paw-bullet" style={style}><span /></span>;
}

export function Purr({ style }: { style?: React.CSSProperties }) {
  return (
    <span className="purr" style={style} aria-label="loading">
      <span /><span /><span /><span /><span /><span />
    </span>
  );
}
