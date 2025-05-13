import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { AddToCart } from '../cart/add-to-cart';
import { AddToCartFragment } from '../cart/fragments';
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

export async function ProductDescription({
  data
}: {
  data: FragmentOf<typeof ProductDescriptionFragment> | {};
}) {
  const product = readFragment(ProductDescriptionFragment, data as any);
  const [name, description, price] = await Promise.all([
    product.name,
    product.description,
    product.price
  ]);
  return (
    <>
      <div className="mb-6 flex flex-col border-b pb-6 dark:border-neutral-700">
        <h1 className="mb-2 text-5xl font-medium">{name}</h1>
        <div className="mr-auto w-auto rounded-full bg-blue-600 p-2 text-sm text-white">
          <Price price={price} />
        </div>
      </div>

      <p className="mb-6 text-sm leading-tight dark:text-white/[60%]">{description}</p>

      <AddToCart data={product} />
    </>
  );
}
