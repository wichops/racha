export function getLocalDate() {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function setCookie(name, value, options = {}) {
  const defaults = {
    path: '/',
    SameSite: 'Lax',
  };
  const opts = { ...defaults, ...options };
  const cookieString = Object.entries(opts)
    .map(([key, val]) => `${key}=${val}`)
    .join(';');
  document.cookie = `${name}=${value};${cookieString}`;
}
