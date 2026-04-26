import { ProviderConfig, providerDot } from "../services/tauri";

interface Props {
  provider: ProviderConfig;
  size?: number;
}

export default function ProviderIcon({ provider, size = 8 }: Props) {
  return (
    <span
      style={{
        display: "inline-block",
        width: size,
        height: size,
        borderRadius: "50%",
        background: providerDot(provider),
        flexShrink: 0,
      }}
    />
  );
}
