import { Dialog, Transition } from '@headlessui/react';
import { ShoppingCartIcon } from '@heroicons/react/24/outline';
import { FragmentOf, readFragment } from 'gql.tada';
import Image from 'next/image';
import Link from 'next/link';
import { Fragment, useEffect, useRef, useState } from 'react';
import Price from '../price';
import CloseCart from './close-cart';
import { DeleteItemButton } from './delete-item-button';
import { EditItemQuantityButton } from './edit-item-quantity-button';
import { CartItemFragment, CartFragment } from './fragments';
import OpenCart from './open-cart';

function CartItem({
  data,
  i,
  closeCart
}: {
  data: FragmentOf<typeof CartItemFragment>;
  i: number;
  closeCart: () => void;
}) {
  const item = readFragment(CartItemFragment, data);

  const [message, setMessage] = useState<string | undefined>();
  return (
    <li className="flex w-full flex-col border-b border-neutral-300 dark:border-neutral-700">
      <div className="relative flex w-full flex-row justify-between px-1 py-4">
        <div className="absolute z-40 -mt-2 ml-[55px]">
          <DeleteItemButton productId={item.product.id} i={i} />
        </div>
        <Link
          id={`cart-item-${i}`}
          href={`/cached/product/${item.product.slug}`}
          onClick={closeCart}
          className="z-30 flex flex-row space-x-4"
        >
          <div className="relative h-16 w-16 cursor-pointer overflow-hidden rounded-md border border-neutral-300 bg-neutral-300 dark:border-neutral-700 dark:bg-neutral-900 dark:hover:bg-neutral-800">
            <Image
              className="h-full w-full object-cover"
              width={64}
              height={64}
              alt={item.product.coverImage.alt}
              src={item.product.coverImage.src}
            />
          </div>

          <div className="flex flex-1 flex-col text-base">
            <span className="leading-tight">{item.product.name}</span>
          </div>
        </Link>
        <div className="flex h-16 flex-col justify-between">
          <Price
            className="flex justify-end space-y-2 text-right text-sm"
            price={item.quantity * item.product.price}
          />
          <div className="ml-auto flex h-9 flex-row items-center rounded-full border border-neutral-200 dark:border-neutral-700">
            <EditItemQuantityButton
              productId={item.product.id}
              type="minus"
              i={i}
              setMessage={setMessage}
            />
            <p className="w-6 text-center">
              <span className="w-full text-sm">{item.quantity}</span>
            </p>
            <EditItemQuantityButton
              productId={item.product.id}
              type="plus"
              i={i}
              setMessage={setMessage}
            />
          </div>
        </div>
      </div>
      {message && (
        <p aria-hidden className="mb-4 text-red-700">
          {message}
        </p>
      )}
    </li>
  );
}

export default function Cart({ data }: { data: FragmentOf<typeof CartFragment> | null }) {
  const cart = readFragment(CartFragment, data);

  const [isOpen, setIsOpen] = useState(false);
  const currentQuantity = cart?.items.reduce((sum, item) => (sum += item.quantity), 0) ?? 0;
  const quantityRef = useRef(currentQuantity);
  const openCart = () => setIsOpen(true);
  const closeCart = () => setIsOpen(false);

  useEffect(() => {
    // Open cart modal when quantity changes.
    if (currentQuantity !== quantityRef.current) {
      // But only if it's not already open (quantity also changes when editing items in cart).
      if (!isOpen) {
        setIsOpen(true);
      }

      // Always update the quantity reference
      quantityRef.current = currentQuantity;
    }
  }, [isOpen, currentQuantity, quantityRef]);

  if (!cart) return null;

  return (
    <>
      <button id="modal-open" aria-label="Open cart" onClick={openCart}>
        <OpenCart quantity={cart.items.length} />
      </button>
      <Transition show={isOpen}>
        <Dialog onClose={closeCart} className="relative z-50">
          <Transition.Child
            as={Fragment}
            enter="transition-all ease-in-out duration-300"
            enterFrom="opacity-0 backdrop-blur-none"
            enterTo="opacity-100 backdrop-blur-[.5px]"
            leave="transition-all ease-in-out duration-200"
            leaveFrom="opacity-100 backdrop-blur-[.5px]"
            leaveTo="opacity-0 backdrop-blur-none"
          >
            <div className="fixed inset-0 bg-black/30" aria-hidden="true" />
          </Transition.Child>
          <Transition.Child
            as={Fragment}
            enter="transition-all ease-in-out duration-300"
            enterFrom="translate-x-full"
            enterTo="translate-x-0"
            leave="transition-all ease-in-out duration-200"
            leaveFrom="translate-x-0"
            leaveTo="translate-x-full"
          >
            <Dialog.Panel className="fixed bottom-0 right-0 top-0 flex h-full w-full flex-col border-l border-neutral-200 bg-white/80 p-6 text-black backdrop-blur-xl md:w-[390px] dark:border-neutral-700 dark:bg-black/80 dark:text-white">
              <div className="flex items-center justify-between">
                <p className="text-lg font-semibold">My Cart</p>

                <button id="modal-close" aria-label="Close cart" onClick={closeCart}>
                  <CloseCart />
                </button>
              </div>

              {cart.items.length === 0 ? (
                <div className="mt-20 flex w-full flex-col items-center justify-center overflow-hidden">
                  <ShoppingCartIcon className="h-16" />
                  <p className="mt-6 text-center text-2xl font-bold">Your cart is empty.</p>
                </div>
              ) : (
                <div className="flex h-full flex-col justify-between overflow-hidden p-1">
                  <ul className="flex-grow overflow-auto py-4">
                    {cart.items.map((item, i) => (
                      <CartItem key={item.product.slug} data={item} i={i} closeCart={closeCart} />
                    ))}
                  </ul>
                  <div className="py-4 text-sm text-neutral-500 dark:text-neutral-400">
                    <div className="mb-3 flex items-center justify-between border-b border-neutral-200 pb-1 pt-1 dark:border-neutral-700">
                      <p>Total</p>
                      <Price
                        className="text-right text-base text-black dark:text-white"
                        price={cart.totalPrice}
                      />
                    </div>
                  </div>
                  <a
                    href="https://checkout.todo"
                    className="block w-full rounded-full bg-blue-600 p-3 text-center text-sm font-medium text-white opacity-90 hover:opacity-100"
                  >
                    Proceed to Checkout
                  </a>
                </div>
              )}
            </Dialog.Panel>
          </Transition.Child>
        </Dialog>
      </Transition>
    </>
  );
}
