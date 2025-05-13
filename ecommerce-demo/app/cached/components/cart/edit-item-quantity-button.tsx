import { MinusIcon, PlusIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { useEffect } from 'react';
import { useMutation } from 'urql';
import LoadingDots from '../loading-dots';
import { AddToCartMutation, RemoveFromCartMutation } from './mutations';

function SubmitButton({
  type,
  fetching,
  i
}: {
  type: 'plus' | 'minus';
  fetching: boolean;
  i: number;
}) {
  return (
    <button
      id={`edit-item-quantity-${type}-${i}`}
      type="submit"
      onClick={(e: React.FormEvent<HTMLButtonElement>) => {
        if (fetching) e.preventDefault();
      }}
      aria-label={type === 'plus' ? 'Increase item quantity' : 'Reduce item quantity'}
      aria-disabled={fetching}
      className={clsx(
        'ease flex h-full min-w-[36px] max-w-[36px] flex-none items-center justify-center rounded-full px-2 transition-all duration-200 hover:border-neutral-800 hover:opacity-80',
        {
          'cursor-not-allowed': fetching,
          'ml-auto': type === 'minus'
        }
      )}
    >
      {fetching ? (
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
  productId,
  type,
  i,
  setMessage
}: {
  productId: string;
  type: 'plus' | 'minus';
  i: number;
  setMessage: (
    // eslint-disable-next-line no-unused-vars
    _message: string | undefined
  ) => void;
}) {
  const [addToCartState, addToCart] = useMutation(AddToCartMutation);
  const [removeFromCartState, removeFromCart] = useMutation(RemoveFromCartMutation);

  useEffect(() => {
    setMessage(
      addToCartState.error?.message ??
        (addToCartState.data && !addToCartState.data.addToCart
          ? 'Must be logged in to edit cart'
          : undefined) ??
        removeFromCartState.error?.message ??
        (removeFromCartState.data && !removeFromCartState.data.removeFromCart
          ? 'Must be logged in to edit cart'
          : undefined) ??
        undefined
    );
  }, [
    setMessage,
    addToCartState.error,
    addToCartState.data,
    removeFromCartState.error,
    removeFromCartState.data
  ]);

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (type === 'plus') {
          addToCart({ productId, quantity: 1 });
        } else {
          removeFromCart({ productId, quantity: 1 });
        }
      }}
    >
      <SubmitButton
        type={type}
        fetching={addToCartState.fetching || removeFromCartState.fetching}
        i={i}
      />
      <p aria-live="polite" className="sr-only" role="status">
        {addToCartState.error?.message ?? removeFromCartState.error?.message}
      </p>
    </form>
  );
}
