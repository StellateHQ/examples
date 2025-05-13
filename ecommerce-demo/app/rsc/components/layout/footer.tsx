export default function Footer() {
  return (
    <footer className="text-sm text-neutral-500 dark:text-neutral-400">
      <div className="border-t border-neutral-200 py-6 text-sm dark:border-neutral-700">
        <div className="mx-auto flex w-full max-w-7xl flex-col items-center gap-1 px-4 md:flex-row md:gap-0 md:px-4 min-[1320px]:px-0">
          <p>&copy; {new Date().getFullYear()} ACME Inc. All rights reserved.</p>
          <p className="md:ml-auto">
            <a href="https://stellate.co" className="text-black dark:text-white">
              Created by Stellate
            </a>
          </p>
        </div>
      </div>
    </footer>
  );
}
