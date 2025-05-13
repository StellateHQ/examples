'use client';

import { useEffect } from 'react';

export function ClickCapture() {
  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      if (!(e.target instanceof Element)) return;

      let element = e.target;
      while (element.parentElement) {
        if (element.id) {
          window.top?.postMessage({ source: 'stellate-demo-graphql-server', click: element.id });
          return;
        }
        element = element.parentElement;
      }
    };

    // We use mousedown instead of click because the cart dialog stops propagation of click events
    window.addEventListener('mousedown', onClick);
    return () => window.removeEventListener('mousedown', onClick);
  }, []);

  return null;
}
