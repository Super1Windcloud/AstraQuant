import { Skeleton } from "@/components/ui/skeleton"
import { TableBody, TableCell, TableRow } from "@/components/ui/table"

interface MarketTableSkeletonBodyProps {
  rowCount?: number
}

export function MarketTableSkeletonBody({ rowCount = 6 }: MarketTableSkeletonBodyProps) {
  return (
    <TableBody>
      {Array.from({ length: rowCount }, (_, index) => (
        <TableRow key={index} className="border-[#2f3338] hover:bg-transparent">
          <TableCell className="px-0 py-0">
            <div className="grid min-h-[44px] grid-cols-[44px_minmax(0,1fr)] items-center gap-0 pl-4">
              <div className="flex items-center justify-center">
                <Skeleton className="size-6 rounded-full bg-[#2a2d31]" />
              </div>
              <div className="min-w-0 py-3">
                <div className="flex items-center gap-3">
                  <Skeleton className="h-6 w-14 rounded bg-[#2a2d31]" />
                  <Skeleton
                    className="h-4 rounded bg-[#2a2d31]"
                    style={{ width: `${160 + (index % 3) * 36}px` }}
                  />
                </div>
              </div>
            </div>
          </TableCell>
          <TableCell className="px-3 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-24 rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
          <TableCell className="px-3 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-16 rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
          <TableCell className="px-3 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-20 rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
          <TableCell className="px-3 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-20 rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
          <TableCell className="px-3 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-20 rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
          <TableCell className="px-4 py-3 text-right">
            <div className="flex justify-end">
              <Skeleton className="h-4 w-[72px] rounded bg-[#2a2d31]" />
            </div>
          </TableCell>
        </TableRow>
      ))}
    </TableBody>
  )
}
