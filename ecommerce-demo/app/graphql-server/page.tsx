import { graphql } from 'gql.tada';
import { gql } from 'lib/gql';
import { Suspense } from 'react';
import { Carousel, CarouselFragment } from './components/carousel';
import { ThreeItemGrid, ThreeItemGridFragment } from './components/grid/three-items';
import Footer from './components/layout/footer';

const HomePageQuery = graphql(
  `
    query HomePageQuery {
      ...ThreeItemGrid
      ...CarouselFragment @defer
    }
  `,
  [ThreeItemGridFragment, CarouselFragment]
);

export default async function HomePage() {
  const { data } = await gql(HomePageQuery, {});
  if (!data) return null;
  return (
    <>
      <ThreeItemGrid data={data} />
      <Suspense>
        <Carousel data={data} />
      </Suspense>
      <Footer />
    </>
  );
}
