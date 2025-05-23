import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { gql } from 'lib/gql';
import { notFound } from 'next/navigation';
import { Suspense } from 'react';
import Footer from '../../components/layout/footer';
import {
  ProductDescription,
  ProductDescriptionFragment
} from '../../components/product/description';
import { GalleryFragment } from '../../components/product/fragments';
import { Gallery } from '../../components/product/gallery';
import { ProductImage, ProductImageFragment } from '../../components/product/image';
import LoadingDots from '../../components/loading-dots';

const RecommendationsFragment = graphql(
  `
    fragment RecommendationsFragment on Product {
      recommendations {
        id
        slug
        ...ProductImageFragment @defer
      }
    }
  `,
  [ProductImageFragment]
);

async function Recommendations({
  data
}: {
  data: FragmentOf<typeof RecommendationsFragment> | {};
}) {
  const recommendations = await readFragment(RecommendationsFragment, data as any).recommendations;

  if (!recommendations.length) return null;

  return (
    <div className="py-8">
      <h2 className="mb-4 text-2xl font-bold">Recommendations</h2>
      <ul className="flex w-full gap-4 overflow-x-auto pt-1">
        {recommendations.map((product: any, i: any) => (
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
        ...GalleryFragment @defer
        ...ProductDescriptionFragment @defer
        ...RecommendationsFragment @defer
      }
    }
  `,
  [GalleryFragment, ProductDescriptionFragment, RecommendationsFragment]
);

export default async function ProductPage({ params }: { params: { slug: string } }) {
  const { data } = await gql(ProductPageQuery, { slug: params.slug });

  if (!data?.productBySlug) return notFound();

  return (
    <>
      <div className="mx-auto max-w-screen-2xl px-4">
        <div className="flex flex-col rounded-lg border border-neutral-200 bg-white p-8 md:p-12 lg:flex-row lg:gap-8 dark:border-neutral-800 dark:bg-black">
          <div className="h-full w-full basis-full lg:basis-4/6">
            <Suspense
              fallback={
                <div className="relative aspect-square h-full max-h-[550px] w-full overflow-hidden" />
              }
            >
              <Gallery data={data.productBySlug} />
            </Suspense>
          </div>

          <div className="basis-full lg:basis-2/6">
            <ProductDescription data={data.productBySlug} />
          </div>
        </div>
        <Suspense
          fallback={
            <div className="flex h-full min-h-96 items-center justify-center">
              <LoadingDots className="h-8 w-8 bg-black dark:bg-white" />
            </div>
          }
        >
          <Recommendations data={data.productBySlug} />
        </Suspense>
      </div>
      <Footer />
    </>
  );
}
