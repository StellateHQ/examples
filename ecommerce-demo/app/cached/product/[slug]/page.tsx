'use client';

import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { notFound } from 'next/navigation';
import { useMemo } from 'react';
import { useQuery } from 'urql';
import { CartFragment } from '../../components/cart/fragments';
import Footer from '../../components/layout/footer';
import Navbar from '../../components/layout/navbar';
import {
  ProductDescription,
  ProductDescriptionFragment
} from '../../components/product/description';
import { Gallery, GalleryFragment } from '../../components/product/gallery';
import { ProductImage, ProductImageFragment } from '../../components/product/image';

const RecommendationsFragment = graphql(
  `
    fragment RecommendationsFragment on Product {
      recommendations {
        id
        slug
        ...ProductImageFragment
      }
    }
  `,
  [ProductImageFragment]
);

function Recommendations({ data }: { data: FragmentOf<typeof RecommendationsFragment> }) {
  const { recommendations } = readFragment(RecommendationsFragment, data);

  if (!recommendations.length) return null;

  return (
    <div className="py-8">
      <h2 className="mb-4 text-2xl font-bold">Recommendations</h2>
      <ul className="flex w-full gap-4 overflow-x-auto pt-1">
        {recommendations.map((product, i) => (
          <li
            key={product.slug}
            className="aspect-square w-full flex-none min-[475px]:w-1/2 sm:w-1/3 md:w-1/4 lg:w-1/5"
          >
            <ProductImage
              linkId={`recommendations-${i}`}
              data={product}
              fill
              sizes="(min-width: 1024px) 20vw, (min-width: 768px) 25vw, (min-width: 640px) 33vw, (min-width: 475px) 50vw, 100vw"
            />
          </li>
        ))}
      </ul>
    </div>
  );
}

const ProductPageQuery = graphql(
  `
    query ProductPageQuery($slug: String!) {
      productBySlug(slug: $slug) {
        ...GalleryFragment
        ...ProductDescriptionFragment
        ...RecommendationsFragment
      }
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
  [GalleryFragment, ProductDescriptionFragment, RecommendationsFragment, CartFragment]
);

export default function ProductPage({ params }: { params: { slug: string } }) {
  const [{ data, fetching }] = useQuery({
    query: ProductPageQuery,
    variables: useMemo(() => ({ slug: params.slug }), [params.slug])
  });

  if (fetching) return null;
  if (!data?.productBySlug) return notFound();

  return (
    <>
      <Navbar data={'currentUser' in data ? data.currentUser?.cart ?? null : null} />
      <main>
        <div className="mx-auto max-w-screen-2xl px-4">
          <div className="flex flex-col rounded-lg border border-neutral-200 bg-white p-8 md:p-12 lg:flex-row lg:gap-8 dark:border-neutral-800 dark:bg-black">
            <div className="h-full w-full basis-full lg:basis-4/6">
              <Gallery data={data.productBySlug} />
            </div>

            <div className="basis-full lg:basis-2/6">
              <ProductDescription data={data.productBySlug} />
            </div>
          </div>
          <Recommendations data={data.productBySlug} />
        </div>
        <Footer />
      </main>
    </>
  );
}
