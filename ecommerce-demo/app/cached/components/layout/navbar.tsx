import { FragmentOf } from 'gql.tada';
import Link from 'next/link';
import Cart from '../cart';
import LogoSquare from '../logo-square';
import { CartFragment } from '../cart/fragments';

const { SITE_NAME } = process.env;

export default function Navbar({ data }: { data: FragmentOf<typeof CartFragment> | null }) {
  return (
    <nav className="relative flex items-center justify-between p-4 lg:px-6">
      <div className="flex w-full items-center">
        <div className="flex w-full md:w-1/2">
          <Link
            id="navbar"
            href="/cached"
            className="mr-2 flex w-full items-center justify-center md:w-auto lg:mr-6"
          >
            <LogoSquare />
            <div className="ml-2 flex-none text-sm font-medium uppercase md:hidden lg:block">
              {SITE_NAME}
            </div>
          </Link>
        </div>
        <div className="flex justify-end md:w-1/2">
          <Cart data={data} />
        </div>
      </div>
    </nav>
  );
}
