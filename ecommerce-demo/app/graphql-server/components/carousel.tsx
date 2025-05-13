import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { ProductImage, ProductImageFragment } from './product/image';

export const CarouselFragment = graphql(
  `
    fragment CarouselFragment on Query {
      carouselProducts: products(first: 20, offset: 3) {
        id
        slug
        ...ProductImageFragment @defer
      }
    }
  `,
  [ProductImageFragment]
);

export async function Carousel({ data }: { data: FragmentOf<typeof CarouselFragment> | {} }) {
  const carouselProducts = await readFragment(CarouselFragment, data as any).carouselProducts;

  if (!carouselProducts.length) return null;

  return (
    <div className=" w-full overflow-x-auto pb-6 pt-1">
      <ul className="flex animate-carousel gap-4">
        {carouselProducts.map((product: any) => (
          <li
            key={`${product.slug}`}
            className="relative aspect-square h-[30vh] max-h-[240px] w-2/3 max-w-[320px] flex-none md:w-1/3"
          >
            <ProductImage
              linkId={`carousel-${product.slug}`}
              data={product}
              fill
              sizes="(min-width: 1024px) 25vw, (min-width: 768px) 33vw, 50vw"
            />
          </li>
        ))}
      </ul>
    </div>
  );
}
