'use client';

import { MinusIcon, PlusIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { useEffect } from 'react';
import { useFormState, useFormStatus } from 'react-dom';
import { updateItemQuantity } from '../cart/actions';
import LoadingDots from '../loading-dots';

function SubmitButton({ type, i }: { type: 'plus' | 'minus'; i: number }) {
  const { pending } = useFormStatus();

  return (
    <button
      id={`edit-item-quantity-${type}-${i}`}
      type="submit"
      onClick={(e: React.FormEvent<HTMLButtonElement>) => {
        if (pending) e.preventDefault();
      }}
      aria-label={type === 'plus' ? 'Increase item quantity' : 'Reduce item quantity'}
      aria-disabled={pending}
      className={clsx(
        'ease flex h-full min-w-[36px] max-w-[36px] flex-none items-center justify-center rounded-full px-2 transition-all duration-200 hover:border-neutral-800 hover:opacity-80',
        {
          'cursor-not-allowed': pending,
          'ml-auto': type === 'minus'
        }
      )}
    >
      {pending ? (
        <LoadingDots className="bg-black dark:bg-white" />
      ) : type === 'plus' ? (
        <PlusIcon className="h-4 w-4 dark:text-neutral-500" />
      ) : (
        <MinusIcon className="h-4 w-4 dark:text-neutral-500" />
      )}
    </button>
  );
}

export function EditItemQuantityButton({
  quantity,
  productId,
  type,
  i,
  setMessage
}: {
  quantity: number;
  productId: string;
  type: 'plus' | 'minus';
  i: number;
  setMessage: (
    // eslint-disable-next-line no-unused-vars
    _message: string | undefined
  ) => void;
}) {
  const [message, formAction] = useFormState(updateItemQuantity, null);
  const payload = {
    productId,
    quantity: type === 'plus' ? quantity + 1 : quantity - 1
  };
  const actionWithVariant = formAction.bind(null, payload);

  useEffect(() => {
    setMessage(message ?? undefined);
  }, [setMessage, message]);

  return (
    <form action={actionWithVariant}>
      <SubmitButton type={type} i={i} />
      <p aria-live="polite" className="sr-only" role="status">
        {message}
      </p>
    </form>
  );
}
