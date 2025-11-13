import _init, { search as _search } from './pkg/docfind.js';

let didInit = false;

export function init() {
  return _init();
}

export default async function search(needle, maxResults) {
  if (!didInit) {
    await _init();
    didInit = true;
  }
  return _search(needle, maxResults);
}