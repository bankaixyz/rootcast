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

  const date = parseTimestamp(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
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

function parseTimestamp(value: string) {
  const utcTimestamp = value.match(
    /^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})(?::(\d{2}))?$/,
  );
  if (utcTimestamp) {
    const [, year, month, day, hour, minute, second = "00"] = utcTimestamp;
    return new Date(
      Date.UTC(
        Number(year),
        Number(month) - 1,
        Number(day),
        Number(hour),
        Number(minute),
        Number(second),
      ),
    );
  }

  return new Date(value);
}
