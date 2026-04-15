export function normalizeId(id: bigint): string {
  const str = id.toString();
  const padded = str.length >= 5 ? str : str.padStart(5, '0');
  return `#${padded}`;
}
