import Link from 'next/link';
import { Suspense } from 'react';
import Cart from '../cart';
import OpenCart from '../cart/open-cart';
import LogoSquare from '../logo-square';

const { SITE_NAME } = process.env;

export default function Navbar() {
  return (
    <nav className="relative flex items-center justify-between p-4 lg:px-6">
      <div className="flex w-full items-center">
        <div className="flex w-full md:w-1/2">
          <Link
            id="navbar"
            href="/rsc"
            className="mr-2 flex w-full items-center justify-center md:w-auto lg:mr-6"
          >
            <LogoSquare />
            <div className="ml-2 flex-none text-sm font-medium uppercase md:hidden lg:block">
              {SITE_NAME}
            </div>
          </Link>
        </div>
        <div className="flex justify-end md:w-1/2">
          <Suspense fallback={<OpenCart />}>
            <Cart />
          </Suspense>
        </div>
      </div>
    </nav>
  );
}
