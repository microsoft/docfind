import init, { search as _search } from './pkg/docfind.js';

let didInit = false;
export default async function search(needle) {
  if (!didInit) {
    await init();
    didInit = true;
  }
  return _search(needle);
}