export function shortHash(value: string, head = 10, tail = 8) {
  if (value.length <= head + tail + 3) {
    return value;
  }

  return `${value.slice(0, head)}...${value.slice(-tail)}`;
}

export function formatBlock(value: number) {
  return new Intl.NumberFormat("en-US").format(value);
}

export function formatTimestamp(value: string | null | undefined) {
  if (!value) {
    return "Pending";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export function formatAgeSeconds(value: number | null) {
  if (value === null) {
    return "No proof requests yet";
  }

  if (value < 60) {
    return `${value}s ago`;
  }

  const minutes = Math.floor(value / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }

  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}
