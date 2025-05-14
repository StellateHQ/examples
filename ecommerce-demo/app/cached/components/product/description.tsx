import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { AddToCart, AddToCartFragment } from '../cart/add-to-cart';
import Price from '../price';

export const ProductDescriptionFragment = graphql(
  `
    fragment ProductDescriptionFragment on Product {
      name
      description
      price
      ...AddToCartFragment
    }
  `,
  [AddToCartFragment]
);

export function ProductDescription({
  data
}: {
  data: FragmentOf<typeof ProductDescriptionFragment>;
}) {
  const product = readFragment(ProductDescriptionFragment, data);
  return (
    <>
      <div className="mb-6 flex flex-col border-b pb-6 dark:border-neutral-700">
        <h1 className="mb-2 text-5xl font-medium">{product.name}</h1>
        <div className="mr-auto w-auto rounded-full bg-blue-600 p-2 text-sm text-white">
          <Price price={product.price} />
        </div>
      </div>

      <p className="mb-6 text-sm leading-tight dark:text-white/[60%]">{product.description}</p>

      <AddToCart data={product} />
    </>
  );
}
