'use client';

import { graphql } from 'gql.tada';
import { useQuery } from 'urql';
import { Carousel, CarouselFragment } from './components/carousel';
import { CartFragment } from './components/cart/fragments';
import { ThreeItemGrid, ThreeItemGridFragment } from './components/grid/three-items';
import Footer from './components/layout/footer';
import Navbar from './components/layout/navbar';

const HomePageQuery = graphql(
  `
    query HomePageQuery {
      ...ThreeItemGrid
      ...CarouselFragment
      ... @defer {
        currentUser {
          id
          cart {
            ...CartFragment
          }
        }
      }
    }
  `,
  [ThreeItemGridFragment, CarouselFragment, CartFragment]
);

export default function HomePage() {
  const [{ data }] = useQuery({ query: HomePageQuery });
  if (!data) return null;
  return (
    <>
      <Navbar data={'currentUser' in data ? (data.currentUser?.cart ?? null) : null} />
      <main>
        <ThreeItemGrid data={data} />
        <Carousel data={data} />
        <Footer />
      </main>
    </>
  );
}
