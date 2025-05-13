'use client';

import { PlusIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { useFormState, useFormStatus } from 'react-dom';
import { FragmentOf, readFragment } from 'gql.tada';
import { use } from 'react';
import LoadingDots from '../loading-dots';
import { addToCart } from './actions';
import { AddToCartFragment } from './fragments';

function SubmitButton({ availableForSale }: { availableForSale: boolean }) {
  const { pending } = useFormStatus();
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
        if (pending) e.preventDefault();
      }}
      aria-label="Add to cart"
      aria-disabled={pending}
      className={clsx(buttonClasses, {
        'hover:opacity-90': true,
        disabledClasses: pending
      })}
    >
      <div className="absolute left-0 ml-4">
        {pending ? <LoadingDots className="mb-3 bg-white" /> : <PlusIcon className="h-5" />}
      </div>
      Add To Cart
    </button>
  );
}

export function AddToCart({ data }: { data: FragmentOf<typeof AddToCartFragment> }) {
  const product = readFragment(AddToCartFragment, data);
  const id = use(Promise.resolve(product.id));
  const hasStock = use(Promise.resolve(product.hasStock));

  const [message, formAction] = useFormState(addToCart, null);
  const actionWithVariant = formAction.bind(null, id);

  return (
    <form action={actionWithVariant}>
      <SubmitButton availableForSale={hasStock} />
      <p aria-hidden className="mt-4 text-red-700">
        {message}
      </p>
      <p aria-live="polite" className="sr-only" role="status">
        {message}
      </p>
    </form>
  );
}
