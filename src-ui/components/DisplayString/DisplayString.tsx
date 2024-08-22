import { Fragment } from "react";

const DisplayString = ({ string }: { string: string }) => {
  const formattedCode = string.split("↵").map((line, index) => (
    <Fragment key={index}>
      {line}
      <br />
    </Fragment>
  ));

  return <span style={{ whiteSpace: "pre-wrap" }}>{formattedCode}</span>;
};

export default DisplayString;
