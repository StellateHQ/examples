import { Suspense } from 'react';
import { Carousel } from './components/carousel';
import { ThreeItemGrid } from './components/grid/three-items';
import Footer from './components/layout/footer';

export const runtime = 'edge';

export default async function HomePage() {
  return (
    <>
      <ThreeItemGrid />
      <Suspense>
        <Carousel />
      </Suspense>
      <Footer />
    </>
  );
}
