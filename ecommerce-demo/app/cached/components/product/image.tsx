import clsx from 'clsx';
import Image from 'next/image';
import Link from 'next/link';
import { FragmentOf, graphql, readFragment } from 'gql.tada';
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

export function ProductImage({
  linkId,
  data,
  labelPosition = 'bottom',
  ...props
}: {
  data: FragmentOf<typeof ProductImageFragment>;
  linkId: string;
  labelPosition?: 'bottom' | 'center';
} & Omit<React.ComponentProps<typeof Image>, 'src' | 'alt'>) {
  const product = readFragment(ProductImageFragment, data);
  return (
    <Link
      id={linkId}
      className="relative block aspect-square h-full w-full"
      href={`/cached/product/${product.slug}`}
    >
      <div className="group relative flex h-full w-full items-center justify-center overflow-hidden rounded-lg border border-neutral-200 bg-white hover:border-blue-600 dark:border-neutral-800 dark:bg-black">
        <Image
          className="relative h-full w-full object-contain transition duration-300 ease-in-out group-hover:scale-105"
          src={product.coverImage.src}
          alt={product.name}
          {...props}
        />
        <div
          className={clsx('absolute bottom-0 left-0 flex w-full px-4 pb-4 @container/label', {
            'lg:px-20 lg:pb-[35%]': labelPosition === 'center'
          })}
        >
          <div className="flex items-center rounded-full border bg-white/70 p-1 text-xs font-semibold text-black backdrop-blur-md dark:border-neutral-800 dark:bg-black/70 dark:text-white">
            <h3 className="mr-4 line-clamp-2 flex-grow pl-2 leading-none tracking-tight">
              {product.name}
            </h3>
            <Price
              className="flex-none rounded-full bg-blue-600 p-2 text-white"
              price={product.price}
              currencyCodeClassName="hidden @[275px]/label:inline"
            />
          </div>
        </div>
      </div>
    </Link>
  );
}
