import clsx from 'clsx';
import Image from 'next/image';
import Link from 'next/link';
import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { Suspense } from 'react';
import Price from '../price';

export const ProductImageFragment = graphql(`
  fragment ProductImageFragment on Product {
    slug
    name
    price
    coverImage {
      src
    }
  }
`);

type ProductImageProps = {
  data: FragmentOf<typeof ProductImageFragment> | {};
  linkId: string;
  labelPosition?: 'bottom' | 'center';
} & Omit<React.ComponentProps<typeof Image>, 'src' | 'alt'>;

async function ProductImageInternal({
  data,
  linkId,
  labelPosition = 'bottom',
  ...props
}: ProductImageProps) {
  const product = readFragment(ProductImageFragment, data as any);
  const [slug, name, price, coverImage] = await Promise.all([
    product.slug,
    product.name,
    product.price,
    product.coverImage
  ]);
  return (
    <Link
      id={linkId}
      className="relative block aspect-square h-full w-full"
      href={`/graphql-server/product/${slug}`}
    >
      <div className="group relative flex h-full w-full items-center justify-center overflow-hidden rounded-lg border border-neutral-200 bg-white hover:border-blue-600 dark:border-neutral-800 dark:bg-black">
        <Image
          className="relative h-full w-full object-contain transition duration-300 ease-in-out group-hover:scale-105"
          src={coverImage.src}
          alt={name}
          {...props}
        />
        <div
          className={clsx('absolute bottom-0 left-0 flex w-full px-4 pb-4 @container/label', {
            'lg:px-20 lg:pb-[35%]': labelPosition === 'center'
          })}
        >
          <div className="flex items-center rounded-full border bg-white/70 p-1 text-xs font-semibold text-black backdrop-blur-md dark:border-neutral-800 dark:bg-black/70 dark:text-white">
            <h3 className="mr-4 line-clamp-2 flex-grow pl-2 leading-none tracking-tight">{name}</h3>
            <Price
              className="flex-none rounded-full bg-blue-600 p-2 text-white"
              price={price}
              currencyCodeClassName="hidden @[275px]/label:inline"
            />
          </div>
        </div>
      </div>
    </Link>
  );
}

export function ProductImage(props: ProductImageProps) {
  return (
    <Suspense
      fallback={
        <div
          role="status"
          className="h-full w-full animate-pulse space-y-8 md:flex md:items-center md:space-x-8 md:space-y-0 rtl:space-x-reverse"
        >
          <div className="flex h-full w-full items-center justify-center rounded bg-gray-300 sm:w-96 dark:bg-gray-700">
            <svg
              className="h-10 w-10 text-gray-200 dark:text-gray-600"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="currentColor"
              viewBox="0 0 20 18"
            >
              <path d="M18 0H2a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2Zm-5.5 4a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3Zm4.376 10.481A1 1 0 0 1 16 15H4a1 1 0 0 1-.895-1.447l3.5-7A1 1 0 0 1 7.468 6a.965.965 0 0 1 .9.5l2.775 4.757 1.546-1.887a1 1 0 0 1 1.618.1l2.541 4a1 1 0 0 1 .028 1.011Z" />
            </svg>
          </div>
          <span className="sr-only">Loading...</span>
        </div>
      }
    >
      {<ProductImageInternal {...props} />}
    </Suspense>
  );
}
