'use client';

import { XMarkIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { useFormState, useFormStatus } from 'react-dom';
import { removeFromCart } from '../cart/actions';
import LoadingDots from '../loading-dots';

function SubmitButton({ i }: { i: number }) {
  const { pending } = useFormStatus();

  return (
    <button
      id={`delete-item-${i}`}
      type="submit"
      onClick={(e: React.FormEvent<HTMLButtonElement>) => {
        if (pending) e.preventDefault();
      }}
      aria-label="Remove cart item"
      aria-disabled={pending}
      className={clsx(
        'ease flex h-[17px] w-[17px] items-center justify-center rounded-full bg-neutral-500 transition-all duration-200',
        {
          'cursor-not-allowed px-0': pending
        }
      )}
    >
      {pending ? (
        <LoadingDots className="bg-white" />
      ) : (
        <XMarkIcon className="hover:text-accent-3 mx-[1px] h-4 w-4 text-white dark:text-black" />
      )}
    </button>
  );
}

export function DeleteItemButton({ productId, i }: { productId: string; i: number }) {
  const [message, formAction] = useFormState(removeFromCart, null);
  const actionWithVariant = formAction.bind(null, productId);

  return (
    <form action={actionWithVariant}>
      <SubmitButton i={i} />
      <p aria-hidden className="mt-4 text-red-700">
        {message}
      </p>
      <p aria-live="polite" className="sr-only" role="status">
        {message}
      </p>
    </form>
  );
}
