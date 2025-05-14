import clsx from 'clsx';

const Price = ({
  price,
  className,
  currencyCodeClassName
}: {
  price: number;
  className?: string;
  currencyCodeClassName?: string;
} & React.ComponentProps<'p'>) => (
  <p suppressHydrationWarning={true} className={className}>
    {`${new Intl.NumberFormat(undefined, {
      style: 'currency',
      currency: 'USD',
      currencyDisplay: 'narrowSymbol'
    }).format(price)}`}
    <span className={clsx('ml-1 inline', currencyCodeClassName)}>USD</span>
  </p>
);

export default Price;
