import { ReactNode, Suspense } from 'react';
import { ClickCapture } from './components/click-capture';
import Navbar from './components/layout/navbar';
import LoadingDots from './components/loading-dots';

export default async function RootLayout({ children }: { children: ReactNode }) {
  return (
    <>
      <Navbar />
      <Suspense
        fallback={
          <div className="flex h-full min-h-96 items-center justify-center">
            <LoadingDots className="h-8 w-8 bg-black dark:bg-white" />
          </div>
        }
      >
        <main>{children}</main>
      </Suspense>
      <ClickCapture />
    </>
  );
}
