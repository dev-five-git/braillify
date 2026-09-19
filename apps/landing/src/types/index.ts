export type Merge<T, U> = Omit<T, Extract<keyof T, keyof U>> & U

export type TestStatus = [
  total: number,
  fail: number,
  worldTotal: number,
  worldFail: number,
  jeomsarangTotal: number,
  jeomsarangFail: number,
  Array<
    [
      text: string,
      note: string,
      expected: string,
      actual: string,
      isSuccess: boolean,
      world: string,
      worldIsSuccess: boolean,
      jeomsarang: string,
      jeomsarangIsSuccess: boolean,
    ]
  >,
]

export type TestStatusMap = Record<string, TestStatus>

export interface TestStatusReportPageInfo {
  pageSize: number
  pageCount: number
}

export type TestStatusReportManifest = Record<string, TestStatusReportPageInfo>
