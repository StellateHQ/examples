import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { ProductImage, ProductImageFragment } from '../product/image';

const ThreeItemGridItemFragment = graphql(
  `
    fragment ThreeItemGridItemFragment on Product {
      id
      slug
      ...ProductImageFragment @defer
    }
  `,
  [ProductImageFragment]
);

function ThreeItemGridItem({
  data,
  size,
  priority
}: {
  data: FragmentOf<typeof ThreeItemGridItemFragment>;
  size: 'full' | 'half';
  priority?: boolean;
}) {
  const item = readFragment(ThreeItemGridItemFragment, data);
  return (
    <div
      className={size === 'full' ? 'md:col-span-4 md:row-span-2' : 'md:col-span-2 md:row-span-1'}
    >
      <ProductImage
        linkId={`three-item-grid-item-${item.slug}`}
        data={item}
        labelPosition={size === 'full' ? 'center' : 'bottom'}
        fill
        sizes={
          size === 'full' ? '(min-width: 768px) 66vw, 100vw' : '(min-width: 768px) 33vw, 100vw'
        }
        priority={priority}
      />
    </div>
  );
}

export const ThreeItemGridFragment = graphql(
  `
    fragment ThreeItemGrid on Query {
      highlightedProducts: products(first: 3) {
        ...ThreeItemGridItemFragment
      }
    }
  `,
  [ThreeItemGridItemFragment]
);

export async function ThreeItemGrid({ data }: { data: FragmentOf<typeof ThreeItemGridFragment> }) {
  const { highlightedProducts } = readFragment(ThreeItemGridFragment, data);

  if (!highlightedProducts[0] || !highlightedProducts[1] || !highlightedProducts[2]) return null;

  const [firstProduct, secondProduct, thirdProduct] = highlightedProducts;

  return (
    <section className="mx-auto grid max-w-screen-2xl gap-4 px-4 pb-4 md:grid-cols-6 md:grid-rows-2">
      <ThreeItemGridItem size="full" data={firstProduct} priority={true} />
      <ThreeItemGridItem size="half" data={secondProduct} priority={true} />
      <ThreeItemGridItem size="half" data={thirdProduct} />
    </section>
  );
}
