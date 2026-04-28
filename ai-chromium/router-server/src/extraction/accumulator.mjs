export function makeTextAccumulator() {
  let full = "";
  return {
    append(candidate) {
      if (!candidate) return "";
      if (candidate === full) return "";
      if (candidate.startsWith(full)) {
        const delta = candidate.slice(full.length);
        full = candidate;
        return delta;
      }
      if (full.endsWith(candidate)) return "";
      full += candidate;
      return candidate;
    },
    value() { return full; },
  };
}
