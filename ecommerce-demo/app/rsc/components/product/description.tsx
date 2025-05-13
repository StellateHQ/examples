import { Suspense } from 'react';
import { AddToCart, PartialProductForAddToCart } from '../cart/add-to-cart';
import Price from '../price';

export type PartialProductForDescription = {
  name: string;
  price: number;
  description: string;
} & PartialProductForAddToCart;

export function ProductDescription({ product }: { product: PartialProductForDescription }) {
  return (
    <>
      <div className="mb-6 flex flex-col border-b pb-6 dark:border-neutral-700">
        <h1 className="mb-2 text-5xl font-medium">{product.name}</h1>
        <div className="mr-auto w-auto rounded-full bg-blue-600 p-2 text-sm text-white">
          <Price price={product.price} />
        </div>
      </div>

      <p className="mb-6 text-sm leading-tight dark:text-white/[60%]">{product.description}</p>

      <Suspense fallback={null}>
        <AddToCart product={product} />
      </Suspense>
    </>
  );
}
