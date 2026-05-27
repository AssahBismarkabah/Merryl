export function number(value: number) {
  return value.toFixed(1);
}

export function percent(value: number) {
  return `${(value * 100).toFixed(1)}%`;
}

export function signed(value: number) {
  return value > 0 ? `+${value.toFixed(0)}` : value.toFixed(0);
}

export function toneClass(value: number) {
  if (value > 0) {
    return "positive";
  }
  if (value < 0) {
    return "negative";
  }
  return "neutral";
}
