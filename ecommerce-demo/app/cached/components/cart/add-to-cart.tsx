import { PlusIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { FragmentOf, graphql, readFragment } from 'gql.tada';
import { useMutation } from 'urql';
import LoadingDots from '../loading-dots';
import { AddToCartMutation } from './mutations';

export const AddToCartFragment = graphql(`
  fragment AddToCartFragment on Product {
    id
    hasStock
  }
`);

function SubmitButton({
  availableForSale,
  fetching
}: {
  availableForSale: boolean;
  fetching: boolean;
}) {
  const buttonClasses =
    'relative flex w-full items-center justify-center rounded-full bg-blue-600 p-4 tracking-wide text-white';
  const disabledClasses = 'cursor-not-allowed opacity-60 hover:opacity-60';

  if (!availableForSale) {
    return (
      <button aria-disabled className={clsx(buttonClasses, disabledClasses)}>
        Out Of Stock
      </button>
    );
  }

  return (
    <button
      id="add-to-cart"
      onClick={(e: React.FormEvent<HTMLButtonElement>) => {
        if (fetching) e.preventDefault();
      }}
      aria-label="Add to cart"
      aria-disabled={fetching}
      className={clsx(buttonClasses, {
        'hover:opacity-90': true,
        disabledClasses: fetching
      })}
    >
      <div className="absolute left-0 ml-4">
        {fetching ? <LoadingDots className="mb-3 bg-white" /> : <PlusIcon className="h-5" />}
      </div>
      Add To Cart
    </button>
  );
}

export function AddToCart({ data }: { data: FragmentOf<typeof AddToCartFragment> }) {
  const [{ fetching, error, data: d }, addToCart] = useMutation(AddToCartMutation);

  const product = readFragment(AddToCartFragment, data);

  let message = error?.message;
  if (!message && d && !d.addToCart) {
    message = 'Must be logged in to add products to cart';
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        addToCart({ productId: product.id, quantity: 1 });
      }}
    >
      <SubmitButton fetching={fetching} availableForSale={Boolean(product.hasStock)} />
      {message && (
        <>
          <p aria-hidden className="mt-4 text-red-700">
            {message}
          </p>
          <p aria-live="polite" className="sr-only" role="status">
            {message}
          </p>
        </>
      )}
    </form>
  );
}
