#include "lld/Common/Driver.h"
#include "llvm/Support/raw_ostream.h"
#include <vector>
#include <iostream>

LLD_HAS_DRIVER(coff)
LLD_HAS_DRIVER(elf)
LLD_HAS_DRIVER(macho)

extern "C" {
    int LLDMain(int argc, const char** argv, bool isVerbose, lld::Flavor flavor) {
        std::vector<const char*> args(argv,argv+argc);
        std::string outs;
        std::string errs;
        llvm::raw_string_ostream lldouts(outs);
        llvm::raw_string_ostream llderrs(errs);

        lld::Driver drv;

        switch (flavor) {
            case lld::Flavor::WinLink:
                drv = lld::coff::link;
                break;
            case lld::Flavor::Gnu:
                drv = lld::elf::link;
                break;
            case lld::Flavor::Darwin:
                drv = lld::macho::link;
                break;
        }

        lld::Result success= lld::lldMain(args, lldouts, llderrs,{
            {flavor, drv}
        });
        if (isVerbose) {
            std::cout << "== LLD OUTPUTS ==" << std::endl << (outs.empty() ? "None" : outs) << std::endl;
            std::cerr << "== LLD ERRORS  ==" << std::endl << (errs.empty() ? "None" : errs) << std::endl;
        }
        return success.retCode;
    }
}
