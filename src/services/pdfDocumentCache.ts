import * as pdfjsLib from "pdfjs-dist";

const cache = new Map<string, Promise<pdfjsLib.PDFDocumentProxy>>();
let hitCount = 0;
let missCount = 0;

export function getCachedPdfDocument(
  pdfUrl: string,
  loader?: () => Promise<pdfjsLib.PDFDocumentProxy>,
): Promise<pdfjsLib.PDFDocumentProxy> {
  const existing = cache.get(pdfUrl);
  if (existing) {
    hitCount++;
    return existing;
  }
  missCount++;
  const promise = loader
    ? loader()
    : pdfjsLib.getDocument({ url: pdfUrl }).promise;
  cache.set(pdfUrl, promise);
  // Remove on failure so retry can work
  promise.catch(() => {
    cache.delete(pdfUrl);
  });
  return promise;
}

export function getCachedPdfDocumentFromBytes(
  key: string,
  bytes: Uint8Array,
): Promise<pdfjsLib.PDFDocumentProxy> {
  const existing = cache.get(key);
  if (existing) {
    hitCount++;
    return existing;
  }
  missCount++;
  const promise = pdfjsLib.getDocument({ data: bytes }).promise;
  cache.set(key, promise);
  promise.catch(() => {
    cache.delete(key);
  });
  return promise;
}

export function invalidatePdfCache(pdfUrl: string) {
  cache.delete(pdfUrl);
}

export function clearPdfCache() {
  cache.clear();
  hitCount = 0;
  missCount = 0;
}

export function getPdfCacheStats() {
  return { hitCount, missCount, size: cache.size };
}
