import { XMarkIcon } from '@heroicons/react/24/outline';
import clsx from 'clsx';
import { useMutation } from 'urql';
import LoadingDots from '../loading-dots';
import { RemoveFromCartMutation } from './mutations';

function SubmitButton({ fetching, i }: { fetching: boolean; i: number }) {
  return (
    <button
      id={`delete-item-${i}`}
      type="submit"
      onClick={(e: React.FormEvent<HTMLButtonElement>) => {
        if (fetching) e.preventDefault();
      }}
      aria-label="Remove cart item"
      aria-disabled={fetching}
      className={clsx(
        'ease flex h-[17px] w-[17px] items-center justify-center rounded-full bg-neutral-500 transition-all duration-200',
        {
          'cursor-not-allowed px-0': fetching
        }
      )}
    >
      {fetching ? (
        <LoadingDots className="bg-white" />
      ) : (
        <XMarkIcon className="hover:text-accent-3 mx-[1px] h-4 w-4 text-white dark:text-black" />
      )}
    </button>
  );
}

export function DeleteItemButton({ productId, i }: { productId: string; i: number }) {
  const [{ fetching, error, data }, removeFromCart] = useMutation(RemoveFromCartMutation);

  let message = error?.message;
  if (!message && data && !data.removeFromCart) {
    message = 'Must be logged in to remove products from cart';
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        removeFromCart({ productId });
      }}
    >
      <SubmitButton fetching={fetching} i={i} />
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
