'use client';

import { useEffect, useRef, useState } from 'react';

export default function RootPage() {
  const [expanded, setExpanded] = useState(false);

  const iframeUncached = useRef<HTMLIFrameElement>(null);
  const iframeCached = useRef<HTMLIFrameElement>(null);
  useEffect(() => {
    const onMessage = (e: MessageEvent) => {
      const data = e.data as unknown;
      if (
        !data ||
        typeof data !== 'object' ||
        !('source' in data) ||
        !('click' in data) ||
        typeof data.click !== 'string'
      )
        return;

      if (data.source === 'stellate-demo-uncached') {
        iframeCached.current?.contentWindow?.document.getElementById(data.click)?.click();
      } else if (data.source === 'stellate-demo-cached') {
        iframeUncached.current?.contentWindow?.document.getElementById(data.click)?.click();
      }
    };

    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, []);

  return (
    <>
      <div className="flex h-screen gap-8 p-8">
        <div className="flex-grow">
          <h1>Uncached</h1>
          <iframe
            ref={iframeUncached}
            className="h-full w-full border-2 border-black"
            src="/uncached"
          />
        </div>
        <div className="flex-grow">
          <h1>Cached</h1>
          <iframe
            ref={iframeCached}
            className="h-full w-full border-2 border-black"
            src="/cached"
          />
        </div>
      </div>

      <div className="fixed bottom-0 right-0 z-50 m-6">
        {expanded ? (
          <div className="mt-4 w-64 rounded-lg border-2 border-black bg-white p-4 shadow-2xl">
            <button
              onClick={() => setExpanded(false)}
              className="absolute right-[-21px] top-0 rounded-full bg-gray-600 px-4 py-2 font-bold text-white hover:bg-gray-700"
            >
              X
            </button>
            <button
              className="rounded-full bg-gray-600 px-4 py-2 font-bold text-white hover:bg-gray-700"
              onClick={() => {
                iframeUncached.current?.contentWindow?.location.reload();
                iframeCached.current?.contentWindow?.location.reload();
              }}
            >
              Reload iframes
            </button>
          </div>
        ) : (
          <button
            onClick={() => setExpanded(true)}
            className="rounded-full bg-gray-600 px-4 py-2 font-bold text-white hover:bg-gray-700"
          >
            Open Debugger
          </button>
        )}
      </div>
    </>
  );
}
