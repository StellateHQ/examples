import { sql } from 'lib/utils';
import { ProductImage } from './product/image';

type SqlResult = {
  id: string;
  slug: string;
};

export async function Carousel() {
  const products = await sql<SqlResult>(
    `
      SELECT id, slug
      FROM products
      LIMIT 20
      OFFSET 3
    `,
    [],
    2000
  );

  if (!products.length) return null;

  return (
    <div className=" w-full overflow-x-auto pb-6 pt-1">
      <ul className="flex animate-carousel gap-4">
        {products.map((product) => (
          <li
            key={`${product.slug}`}
            className="relative aspect-square h-[30vh] max-h-[240px] w-2/3 max-w-[320px] flex-none md:w-1/3"
          >
            <ProductImage
              linkId={`carousel-${product.slug}`}
              productId={product.id}
              fill
              sizes="(min-width: 1024px) 25vw, (min-width: 768px) 33vw, 50vw"
            />
          </li>
        ))}
      </ul>
    </div>
  );
}
