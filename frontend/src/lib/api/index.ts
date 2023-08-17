export function apiUrl(ep: string): URL {
  return new URL(ep, process.env.API_BASE ?? "http://localhost:3000");
}
