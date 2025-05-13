import { USER_COOKIE, sql } from 'lib/utils';
import { cookies } from 'next/headers';
import { notFound } from 'next/navigation';
import { Suspense } from 'react';
import Footer from '../../components/layout/footer';
import LoadingDots from '../../components/loading-dots';
import { ProductDescription } from '../../components/product/description';
import { Gallery } from '../../components/product/gallery';
import { ProductImage } from '../../components/product/image';

export const runtime = 'edge';

type SqlResultProduct = {
  id: string;
  name: string;
  description: string;
  price: number;
  has_stock: number;
  src: string;
  alt: string;
};

type SqlResultImages = {
  src: string;
  alt: string;
};

export default async function ProductPage({ params }: { params: { slug: string } }) {
  const [[product], images] = await Promise.all([
    sql<SqlResultProduct>(
      `
        SELECT products.id, products.name, products.description, products.price, products.has_stock, images.src, images.alt
        FROM products
        JOIN images ON images.src = products.cover_image_src
        WHERE slug = ?
      `,
      [params.slug],
      1000
    ),
    sql<SqlResultImages>(
      `
        SELECT images.src, images.alt
        FROM products_images
        JOIN images ON images.src = products_images.image_src
        JOIN products ON products.id = products_images.product_id
        WHERE products.slug = ?
      `,
      [params.slug],
      1000
    )
  ]);

  if (!product) return notFound();

  const allImages = [
    { src: product.src, altText: product.alt },
    ...images.map((image) => ({ src: image.src, altText: image.alt }))
  ];

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
              <Gallery images={allImages} />
            </Suspense>
          </div>

          <div className="basis-full lg:basis-2/6">
            <ProductDescription product={product} />
          </div>
        </div>
        <Suspense
          fallback={
            <div className="flex h-full min-h-96 items-center justify-center">
              <LoadingDots className="h-8 w-8 bg-black dark:bg-white" />
            </div>
          }
        >
          <Recommendations />
        </Suspense>
      </div>
      <Footer />
    </>
  );
}

type SqlResultRecommendations = {
  id: string;
  slug: string;
};

async function Recommendations() {
  const userId = cookies().get(USER_COOKIE)?.value;
  const recommendations = userId
    ? await sql<SqlResultRecommendations>(
        `
          SELECT products.id, products.slug
          FROM recommendations
          JOIN products ON products.id = recommendations.product_id
          WHERE recommendations.user_id = ?
        `,
        [userId],
        1000
      )
    : await sql<SqlResultRecommendations>(
        `
          SELECT id, slug
          FROM products
          ORDER BY RANDOM()
          LIMIT 4
        `,
        [],
        1000
      );

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
              productId={product.id}
              fill
              sizes="(min-width: 1024px) 20vw, (min-width: 768px) 25vw, (min-width: 640px) 33vw, (min-width: 475px) 50vw, 100vw"
            />
          </li>
        ))}
      </ul>
    </div>
  );
}
