#include "../src/HttpRouter.h"

#include <cassert>
#include <iostream>

void ApiServer() {
    std::cout << "TestPerformance" << std::endl;
    uWS::HttpRouter<int> r;

    r.add({"GET"}, "/*", [](auto *h) {
        return true;
    });

    r.add({"*"}, "/*", [](auto *h) {
        return true;
    });

}

int main() {
    ApiServer();
    return 0;
}
