function App() {
  return (
    <div className="px-2 pb-3 pt-4 h-screen bg-slate-950 text-lime-600 font-['Source_Code_Pro'] relative">
      <div className="border border-lime-600 h-full pt-3">
        <h1 className="px-2 bg-slate-950 absolute right-1/2 translate-x-1/2 top-1">
          Gastronomy Debugger (fibonacci.uplc)
        </h1>
        <div className="grid grid-rows-[max-content_1fr] h-full text-sm">
          <div className="px-2 pt-1 pb-4">
            <div className="text-center">Step 0/448</div>
            <div className="flex justify-between">
              <div>
                Current: <span className="text-blue-600">Compute</span>
              </div>
              <div>
                Budget: <span className="text-blue-600">100 steps</span> (+100){" "}
                <span className="text-blue-600">100 mem</span> (+100)
              </div>
              <div>
                Next: <span className="text-blue-600">Compute</span>
              </div>
            </div>
          </div>
          <div className="grid grid-cols-2 h-full border-t border-lime-600">
            <div className="relative border-r border-lime-600 p-3">
              <h2 className="left-2 -top-3 bg-slate-950 text-blue-600 absolute px-2 z-10">
                Term
              </h2>
            </div>
            <div className="relative">
              <h2 className="left-2 -top-3 bg-slate-950 absolute px-2">
                Context
              </h2>
              <div className="h-full grid grid-rows-2">
                <div className="p-3">dsd</div>
                <div className="p-3 border-t border-lime-600 relative">
                  <h2 className="left-2 -top-3 bg-slate-950 absolute px-2">
                    Env
                  </h2>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="px-2 bg-slate-950 flex absolute right-1/2 translate-x-1/2 bottom-1 gap-2">
        <div>
          <button className="hover:underline">Next</button>{" "}
          <span className="text-blue-600">{"<N>"}</span>
        </div>
        <div>
          <button className="hover:underline">Previous</button>{" "}
          <span className="text-blue-600">{"<P>"}</span>
        </div>
        <div>
          <button className="hover:underline">Quit</button>{" "}
          <span className="text-blue-600">{"<Q>"}</span>
        </div>
      </div>
    </div>
  );
}

export default App;
