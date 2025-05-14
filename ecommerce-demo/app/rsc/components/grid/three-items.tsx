import { sql } from 'lib/utils';
import { ProductImage } from '../product/image';

type SqlResult = {
  id: string;
  slug: string;
};

function ThreeItemGridItem({
  item,
  size,
  priority
}: {
  item: SqlResult;
  size: 'full' | 'half';
  priority?: boolean;
}) {
  return (
    <div
      className={size === 'full' ? 'md:col-span-4 md:row-span-2' : 'md:col-span-2 md:row-span-1'}
    >
      <ProductImage
        linkId={`three-item-grid-item-${item.slug}`}
        productId={item.id}
        fill
        sizes={
          size === 'full' ? '(min-width: 768px) 66vw, 100vw' : '(min-width: 768px) 33vw, 100vw'
        }
        priority={priority}
      />
    </div>
  );
}

export async function ThreeItemGrid() {
  const homepageProducts = await sql<SqlResult>(
    `
      SELECT id, slug
      FROM products
      LIMIT 3
    `,
    [],
    1000
  );

  if (!homepageProducts[0] || !homepageProducts[1] || !homepageProducts[2]) return null;

  const [firstProduct, secondProduct, thirdProduct] = homepageProducts;

  return (
    <section className="mx-auto grid max-w-screen-2xl gap-4 px-4 pb-4 md:grid-cols-6 md:grid-rows-2">
      <ThreeItemGridItem size="full" item={firstProduct} priority={true} />
      <ThreeItemGridItem size="half" item={secondProduct} priority={true} />
      <ThreeItemGridItem size="half" item={thirdProduct} />
    </section>
  );
}
