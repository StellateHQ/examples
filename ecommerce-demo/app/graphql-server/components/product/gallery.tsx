'use client';

import { ArrowLeftIcon, ArrowRightIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { FragmentOf, readFragment } from 'gql.tada';
import Image from 'next/image';
import Link from 'next/link';
import { ReadonlyURLSearchParams, usePathname, useSearchParams } from 'next/navigation';
import { use } from 'react';
import { GalleryFragment } from './fragments';

export function Gallery({ data }: { data: FragmentOf<typeof GalleryFragment> | {} }) {
  const product = readFragment(GalleryFragment, data as any);
  const coverImage = use(Promise.resolve(product.coverImage));
  const images = use(Promise.resolve(product.images));

  const allImages = [
    { src: coverImage.src, altText: coverImage.alt },
    ...images.map((image) => ({ src: image.src, altText: image.alt }))
  ];

  const pathname = usePathname();
  const searchParams = useSearchParams();
  const imageSearchParam = searchParams.get('image');
  const imageIndex = imageSearchParam ? parseInt(imageSearchParam) : 0;

  const nextSearchParams = new URLSearchParams(searchParams.toString());
  const nextImageIndex = imageIndex + 1 < allImages.length ? imageIndex + 1 : 0;
  nextSearchParams.set('image', nextImageIndex.toString());
  const nextUrl = createUrl(pathname, nextSearchParams);

  const previousSearchParams = new URLSearchParams(searchParams.toString());
  const previousImageIndex = imageIndex === 0 ? allImages.length - 1 : imageIndex - 1;
  previousSearchParams.set('image', previousImageIndex.toString());
  const previousUrl = createUrl(pathname, previousSearchParams);

  const buttonClassName =
    'h-full px-6 transition-all ease-in-out hover:scale-110 hover:text-black dark:hover:text-white flex items-center justify-center';

  return (
    <>
      <div className="relative aspect-square h-full max-h-[550px] w-full overflow-hidden">
        {allImages[imageIndex] && (
          <Image
            className="h-full w-full object-contain"
            fill
            sizes="(min-width: 1024px) 66vw, 100vw"
            alt={allImages[imageIndex]?.altText as string}
            src={allImages[imageIndex]?.src as string}
            priority={true}
          />
        )}

        {allImages.length > 1 ? (
          <div className="absolute bottom-[15%] flex w-full justify-center">
            <div className="mx-auto flex h-11 items-center rounded-full border border-white bg-neutral-50/80 text-neutral-500 backdrop-blur dark:border-black dark:bg-neutral-900/80">
              <Link
                id="gallery-previous"
                aria-label="Previous product image"
                href={previousUrl}
                className={buttonClassName}
                scroll={false}
              >
                <ArrowLeftIcon className="h-5" />
              </Link>
              <div className="mx-1 h-6 w-px bg-neutral-500"></div>
              <Link
                id="gallery-next"
                aria-label="Next product image"
                href={nextUrl}
                className={buttonClassName}
                scroll={false}
              >
                <ArrowRightIcon className="h-5" />
              </Link>
            </div>
          </div>
        ) : null}
      </div>

      {allImages.length > 1 ? (
        <ul className="my-12 flex items-center justify-center gap-2 overflow-auto py-1 lg:mb-0">
          {allImages.map((image, index) => {
            const isActive = index === imageIndex;
            const imageSearchParams = new URLSearchParams(searchParams.toString());

            imageSearchParams.set('image', index.toString());

            return (
              <li key={image.src} className="h-20 w-20">
                <Link
                  id={`gallery-enlarge-${image.src}`}
                  aria-label="Enlarge product image"
                  href={createUrl(pathname, imageSearchParams)}
                  scroll={false}
                  className="h-full w-full"
                >
                  <div
                    className={clsx(
                      'group flex h-full w-full items-center justify-center overflow-hidden rounded-lg border bg-white hover:border-blue-600 dark:bg-black',
                      {
                        'border-2 border-blue-600': isActive,
                        'border-neutral-200 dark:border-neutral-800': !isActive
                      }
                    )}
                  >
                    <Image
                      className="relative h-full w-full object-contain transition duration-300 ease-in-out group-hover:scale-105"
                      alt={image.altText}
                      src={image.src}
                      width={80}
                      height={80}
                    />
                  </div>
                </Link>
              </li>
            );
          })}
        </ul>
      ) : null}
    </>
  );
}

const createUrl = (pathname: string, params: URLSearchParams | ReadonlyURLSearchParams) => {
  const paramsString = params.toString();
  const queryString = `${paramsString.length ? '?' : ''}${paramsString}`;

  return `${pathname}${queryString}`;
};
